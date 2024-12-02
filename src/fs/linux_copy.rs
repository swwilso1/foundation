use crate::error::FoundationError;
use crate::progressmeter::ProgressMeter;
use nix::unistd::fsync;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task;

const BLOCKSIZE: libc::size_t = 8388608;

/// Asynchronously copy a file from one location to another.
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
pub async fn async_copy(
    src: &Path,
    dest: &Path,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<(), FoundationError> {
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
        }

        // Make sure to sync the writes to the destination.
        if let Err(e) = fsync(dest_fd) {
            return Err(FoundationError::SyncError(format!(
                "Failed to sync data: {}",
                e
            )));
        }

        Ok(())
    })
    .await
    {
        return Err(FoundationError::JoinError(format!(
            "Failed to join async copy work thread: {}",
            e
        )));
    }
    Ok(())
}
