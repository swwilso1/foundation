use crate::error::FoundationError;
use crate::progressmeter::ProgressMeter;
use nix::unistd::fsync;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;
use std::sync::{Arc, Mutex};

const BLOCKSIZE: libc::size_t = 8388608;

/// Synchronously copy a file from one location to another.
///
/// # Arguments
///
/// * `src` - A reference to a Path representing the source file.
/// * `dest` - A reference to a Path representing the destination file.
/// * `meter` - An optional Arc<Mutex<ProgressMeter>>. If provided, the ProgressMeter will be
///   updated with the number of bytes copied.
///
/// # Returns
///
/// A Result containing `()`. If the file is successfully copied, the result will be `Ok(())`.
/// If an error occurs, the result will be `Err(FoundationError)`.
pub fn copy(
    src: &Path,
    dest: &Path,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<(), FoundationError> {
    if !src.exists() {
        return Err(FoundationError::FileNotFound(src.to_path_buf()));
    }

    // Get the number of bytes in the source file.
    let mut src_bytes = std::fs::metadata(src)?.len();

    // Create the destination file.
    let mut dest_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest)?;

    // Get the destination file descriptor. We use this to call fsync to make sure
    // the data is written to disk.
    let dest_fd = dest_file.as_raw_fd();

    let mut src_file = std::fs::File::open(src)?;

    while src_bytes > 0 {
        let mut buffer = vec![0u8; BLOCKSIZE];
        let bytes_read = src_file.read(&mut buffer)?;
        if bytes_read == 0 && src_bytes > 0 {
            continue;
        }

        dest_file.write_all(&buffer[..bytes_read])?;
        dest_file.flush()?;

        if let Some(meter) = &meter {
            if let Ok(mut meter) = meter.lock() {
                meter.increment_by(bytes_read as u64);
                meter.notify(false);
            }
        }

        src_bytes -= bytes_read as u64;
    }

    // Make sure to sync the writes to the destination.
    if let Err(e) = fsync(dest_fd) {
        return Err(FoundationError::SyncError(format!(
            "Failed to sync data: {}",
            e
        )));
    }

    Ok(())
}

pub fn sync(fd: RawFd) -> Result<(), FoundationError> {
    // Make sure to sync the writes to the destination.
    if let Err(e) = fsync(fd) {
        return Err(FoundationError::SyncError(format!(
            "Failed to sync data: {}",
            e
        )));
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

    #[test]
    fn test_copy_basic() {
        let mut src_file = NamedTempFile::new().unwrap();
        write!(src_file, "Hello, world!").unwrap();
        src_file.flush().unwrap();
        let src_path = src_file.path().to_path_buf();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        copy(&src_path, &dest_path, None).unwrap();

        let src_content = fs::read_to_string(&src_path).unwrap();
        let dest_content = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(src_content, dest_content);
        assert_eq!(dest_content, "Hello, world!");
    }

    #[test]
    fn test_copy_creates_destination_file() {
        let mut src_file = NamedTempFile::new().unwrap();
        write!(src_file, "create me").unwrap();
        src_file.flush().unwrap();

        // Destination does not exist yet; copy must create it.
        let dir = TempDir::new().unwrap();
        let dest_path = dir.path().join("new_destination.txt");
        assert!(!dest_path.exists());

        copy(src_file.path(), &dest_path, None).unwrap();

        assert!(dest_path.exists());
        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "create me");
    }

    #[test]
    fn test_copy_empty_file() {
        let src_file = NamedTempFile::new().unwrap();
        // src_file is empty (0 bytes).
        let dir = TempDir::new().unwrap();
        let dest_path = dir.path().join("empty_dest");

        copy(src_file.path(), &dest_path, None).unwrap();

        assert!(dest_path.exists());
        assert_eq!(fs::metadata(&dest_path).unwrap().len(), 0);
    }

    #[test]
    fn test_copy_overwrites_and_truncates_destination() {
        let mut src_file = NamedTempFile::new().unwrap();
        write!(src_file, "short").unwrap();
        src_file.flush().unwrap();

        // Pre-existing destination with longer content that must be truncated.
        let mut dest_file = NamedTempFile::new().unwrap();
        write!(dest_file, "this is a much longer pre-existing content").unwrap();
        dest_file.flush().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        copy(src_file.path(), &dest_path, None).unwrap();

        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "short");
    }

    #[test]
    fn test_copy_binary_integrity_across_block_boundary() {
        // Write a file larger than BLOCKSIZE so the copy loop runs multiple
        // iterations, using non-text bytes to verify exact binary fidelity.
        let total = BLOCKSIZE + BLOCKSIZE / 2 + 12345;
        let mut data = vec![0u8; total];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i % 256) as u8;
        }

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(&data).unwrap();
        src_file.flush().unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        copy(src_file.path(), &dest_path, None).unwrap();

        let copied = fs::read(&dest_path).unwrap();
        assert_eq!(copied.len(), total);
        assert_eq!(copied, data);
    }

    #[test]
    fn test_copy_source_not_found() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("does_not_exist");
        let dest_path = dir.path().join("dest");

        let err = copy(&missing, &dest_path, None).unwrap_err();
        match err {
            FoundationError::FileNotFound(p) => assert_eq!(p, missing),
            other => panic!("expected FileNotFound, got {:?}", other),
        }
        // Destination must not have been created on the error path.
        assert!(!dest_path.exists());
    }

    #[test]
    fn test_copy_updates_progress_meter() {
        let total = BLOCKSIZE + 1000;
        let data = vec![7u8; total];

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(&data).unwrap();
        src_file.flush().unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        // Track the highest percentage reported through the notifier.
        let last_percent = Arc::new(AtomicU64::new(0));
        let last_percent_clone = last_percent.clone();
        let notifier = Box::new(move |percent: u8| {
            last_percent_clone.store(percent as u64, Ordering::SeqCst);
        });

        let meter = Arc::new(Mutex::new(ProgressMeter::new_with_notifier_and_size(
            notifier,
            total as u64,
        )));

        copy(src_file.path(), &dest_path, Some(meter.clone())).unwrap();

        // The whole file was copied, so the meter should have reached 100%.
        assert_eq!(last_percent.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_sync_succeeds_on_valid_fd() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "data to sync").unwrap();
        file.flush().unwrap();

        let fd = file.as_file().as_raw_fd();
        sync(fd).unwrap();
    }

    #[test]
    fn test_sync_fails_on_invalid_fd() {
        // -1 is never a valid file descriptor.
        let err = sync(-1).unwrap_err();
        match err {
            FoundationError::SyncError(_) => {}
            other => panic!("expected SyncError, got {:?}", other),
        }
    }
}
