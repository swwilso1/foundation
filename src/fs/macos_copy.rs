use crate::error::FoundationError;
use crate::fs::copy::sync;
use crate::progressmeter::ProgressMeter;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    F: Fn() -> bool + std::marker::Send + std::marker::Sync + 'static,
{
    if !src.exists() {
        return Err(FoundationError::FileNotFound(src.to_path_buf()));
    }

    // Get the number of bytes in the source file.
    let mut src_bytes = tokio::fs::metadata(src).await?.len();

    // Create the destination file.
    let mut dest_file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest)
        .await?;

    // Get the destination file descriptor. We use this to call fsync to make sure
    // the data is written to disk.
    let dest_fd = dest_file.as_raw_fd();

    let mut src_file = tokio::fs::File::open(src).await?;

    while src_bytes > 0 {
        let mut buffer = vec![0u8; BLOCKSIZE];
        let bytes_read = src_file.read(&mut buffer).await?;
        if bytes_read == 0 && src_bytes > 0 {
            continue;
        }

        dest_file.write_all(&buffer[..bytes_read]).await?;
        dest_file.flush().await?;

        if let Some(meter) = &meter {
            if let Ok(mut meter) = meter.lock() {
                meter.increment_by(bytes_read as u64);
                meter.notify(false);
            }
        }

        src_bytes -= bytes_read as u64;

        if aborter() {
            sync(dest_fd)?;
            return Err(FoundationError::AbortError("Operation aborted".to_string()));
        }
    }

    sync(dest_fd)?;

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
