use crate::error::FoundationError;
use crate::fs::copy::sync;
use crate::progressmeter::ProgressMeter;
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
/// updated with the number of bytes copied.
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
                return Err(FoundationError::CopyFailed(format!(
                    "Error copying file: {}",
                    std::io::Error::last_os_error()
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
    use tempfile::NamedTempFile;

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
            Arc::new(|| {
                let later = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                if later - now > 2 {
                    return true;
                }
                return false;
            }),
            None,
        )
        .await
        .expect_err("Should have generated an error");
    }
}
