use crate::error::FoundationError;
use crate::fs::async_copy;
use crate::progressmeter::ProgressMeter;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { async_copy(src, dest, meter).await })
}
