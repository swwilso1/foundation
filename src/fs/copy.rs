use crate::error::FoundationError;
use crate::progressmeter::ProgressMeter;
use nix::unistd::fsync;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
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
/// updated with the number of bytes copied.
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
