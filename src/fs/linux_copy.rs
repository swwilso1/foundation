use crate::error::FoundationError;
use crate::fs::copy::sync;
use crate::progressmeter::ProgressMeter;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::task;

const BLOCKSIZE: libc::size_t = 8388608;

/// Asynchronously copy a file from one location to another.
///
/// # Arguments
///
/// * `src` - A reference to a Path representing the source file.
/// * `dest` - A reference to a Path representing the destination file.
/// * `aborter` - A function that returns true if the copy should abort and false otherwise.
/// * `meter` - An optional Arc<Mutex<ProgressMeter>>. If provided, the ProgressMeter will be
///   updated with the number of bytes copied.
///
/// # Returns
///
/// A Result containing `()`. If the file is successfully copied, the result will be `Ok(())`.
/// If an error occurs, the result will be `Err(FoundationError)`.
pub async fn async_copy<F>(
    src: &Path,
    dest: &Path,
    aborter: Arc<F>,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<(), FoundationError>
where
    F: Fn() -> bool + std::marker::Sync + std::marker::Send + 'static,
{
    if !src.exists() {
        return Err(FoundationError::FileNotFound(src.to_path_buf()));
    }

    let src_file = tokio::fs::File::open(src).await?;
    let dest_file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest)
        .await?;

    let src_fd = src_file.as_raw_fd();
    let dest_fd = dest_file.as_raw_fd();

    let metadata = src.metadata()?;
    let mut bytes_still_to_transfer = metadata.len() as libc::size_t;

    let aborted = Arc::new(AtomicBool::new(false));
    let aborted_copy = aborted.clone();

    if let Err(e) = task::spawn_blocking(move || {
        while bytes_still_to_transfer > 0 {
            let bytes_to_transfer = if bytes_still_to_transfer >= BLOCKSIZE {
                BLOCKSIZE
            } else {
                bytes_still_to_transfer
            };

            let bytes_sent =
                unsafe { libc::sendfile(dest_fd, src_fd, std::ptr::null_mut(), bytes_to_transfer) };

            if bytes_sent < 0 {
                let err = io::Error::last_os_error();
                let errno = err.raw_os_error().unwrap_or(0);

                let errmsg = match errno {
                    libc::EIO => {
                        // Hardware I/O error - USB transport failure
                        // Do not retry, the device is in an unknown state.
                        "USB transport failure".to_string()
                    }
                    libc::ENODEV => {
                        // Device is completely gone.
                        "device disappeared from USB subsystem".to_string()
                    }
                    libc::ENOSPC => {
                        // No space left on device.
                        "no space left on device".to_string()
                    }
                    libc::EINTR => {
                        // Interrupted by OS signal, we can safely retry.
                        continue;
                    }
                    libc::EAGAIN => {
                        // The device would block. We probably are not in a non-blocking
                        // case, but give the device a few milliseconds and try again.
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                    libc::EBADF => {
                        // The file descriptor is not invalid
                        // device was lost
                        "invalid file descriptor".to_string()
                    }
                    libc::EFAULT => {
                        // Bad offset pointer to libc::sendfile(). This should not happen.
                        "invalid sendfile offset parameter".to_string()
                    }
                    _ => {
                        // Unknown error
                        format!("sendfile failed with error {}: {}", errno, err)
                    }
                };

                return Err(FoundationError::CopyFailed(format!(
                    "Error copying file: {}",
                    errmsg
                )));
            }

            if bytes_sent == 0 {
                // sendfile returns 0 at end-of-file. The loop condition guarantees the byte
                // counter (taken from the file metadata before the loop) still expects more
                // data, so the source file must have shrunk since the metadata snapshot.
                // Retrying would return 0 forever and spin this loop hot; fail instead.
                return Err(FoundationError::CopyFailed(format!(
                    "Source file ended {} bytes short of its reported size",
                    bytes_still_to_transfer
                )));
            }

            bytes_still_to_transfer -= bytes_sent as libc::size_t;

            if let Some(meter) = &meter {
                if let Ok(mut meter) = meter.lock() {
                    meter.increment_by(bytes_sent as u64);
                    meter.notify(false);
                }
            }

            if aborter() {
                aborted_copy.store(true, std::sync::atomic::Ordering::SeqCst);
                sync(dest_fd)?;
                return Err(FoundationError::AbortError("Operation aborted".to_string()));
            }
        }

        sync(dest_fd)?;

        Ok(())
    })
    .await
    {
        return Err(FoundationError::JoinError(format!(
            "Failed to join async copy work thread: {}",
            e
        )));
    }

    if aborted.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(FoundationError::AbortError("Operation aborted".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_async_copy_source_not_found() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("nope");
        let dest_path = dir.path().join("dest");

        let err = async_copy(&missing, &dest_path, Arc::new(|| false), None)
            .await
            .unwrap_err();
        match err {
            FoundationError::FileNotFound(p) => assert_eq!(p, missing),
            other => panic!("expected FileNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_async_copy_updates_progress_meter() {
        let data = vec![3u8; BLOCKSIZE + 4096];
        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(&data).unwrap();
        src_file.flush().unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        let last_percent = Arc::new(AtomicU64::new(0));
        let last_percent_clone = last_percent.clone();
        let notifier = Box::new(move |percent: u8| {
            last_percent_clone.store(percent as u64, Ordering::SeqCst);
        });
        let meter = Arc::new(Mutex::new(ProgressMeter::new_with_notifier_and_size(
            notifier,
            data.len() as u64,
        )));

        async_copy(src_file.path(), &dest_path, Arc::new(|| false), Some(meter))
            .await
            .unwrap();

        assert_eq!(fs::read(&dest_path).unwrap(), data);
        assert_eq!(last_percent.load(Ordering::SeqCst), 100);
    }

    #[tokio::test]
    async fn test_async_copy() {
        let mut src_file = NamedTempFile::new().unwrap();
        writeln!(src_file, "Hello, world!").unwrap();
        let src_path = src_file.path().to_path_buf();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        async_copy(&src_path, &dest_path, Arc::new(|| false), None)
            .await
            .unwrap();

        let src_content = fs::read_to_string(src_path).unwrap();
        let dest_content = fs::read_to_string(dest_path).unwrap();

        assert_eq!(src_content, dest_content);
    }

    #[tokio::test]
    async fn test_async_copy_empty_file() {
        // A zero-byte source means the sendfile loop never executes; the destination must still be
        // created (and truncated) so that it ends up as an empty file.
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_path_buf();

        let dir = TempDir::new().unwrap();
        let dest_path = dir.path().join("empty_dest");

        async_copy(&src_path, &dest_path, Arc::new(|| false), None)
            .await
            .unwrap();

        assert!(dest_path.exists());
        assert_eq!(fs::metadata(&dest_path).unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_async_copy_truncates_existing_destination() {
        // The destination is opened with truncate(true), so pre-existing longer content must be
        // replaced entirely by the (shorter) source content.
        let mut src_file = NamedTempFile::new().unwrap();
        write!(src_file, "short").unwrap();
        src_file.flush().unwrap();

        let mut dest_file = NamedTempFile::new().unwrap();
        write!(dest_file, "this is a much longer pre-existing payload").unwrap();
        dest_file.flush().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        async_copy(src_file.path(), &dest_path, Arc::new(|| false), None)
            .await
            .unwrap();

        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "short");
    }

    #[tokio::test]
    async fn test_async_copy_with_aborter() {
        let mut src_file = NamedTempFile::new().unwrap();
        let lorem_ipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\
            Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,\
            quis nostrum exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. \
            Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu\
            fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa\
            qui officia deserunt mollit anim id est laborum.";

        for _ in 1..100000 {
            writeln!(src_file, "{}", lorem_ipsum).unwrap();
        }

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        async_copy(
            src_file.path(),
            &dest_path,
            Arc::new(move || {
                let later = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                later - now > 2
            }),
            None,
        )
        .await
        .expect_err("Should have generated an error");
    }
}
