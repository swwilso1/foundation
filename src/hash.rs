//! The `hash` module provides synchronous and asynchronous functions to hash files and directories.p
//! The functions in this module use the `blake3` crate to hash files and directories. The asynchronous
//! functions use the `tokio` crate to perform the asynchronous operations.

use crate::error::FoundationError;
use crate::progressmeter::ProgressMeter;
use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::{
    fs::File as TokioFile,
    io::{AsyncReadExt, BufReader as TokioBufReader},
};

pub use blake3::Hasher;

const CHUNK_SIZE: usize = 1024 * 1024;

/// Get the hash of a file, optionally reporting progress to a ProgressMeter.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `meter` - An optional reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing a string. If the file is successfully hashed, the result will be `Ok(String)`.
pub fn get_hash_for_file(
    path: &Path,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError> {
    let mut file = StdFile::open(path)?;
    let metadata = file.metadata()?;
    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut hasher = Hasher::new();

    let mut left_to_read = metadata.len();
    while left_to_read > 0 {
        let bytes_to_read = std::cmp::min(CHUNK_SIZE as u64, left_to_read) as usize;
        let bytes_read = file.read(&mut chunk[..bytes_to_read])?;
        left_to_read -= bytes_read as u64;
        hasher.update(&chunk[..bytes_read]);
        if let Some(meter) = &meter {
            if let Ok(mut meter) = meter.lock() {
                meter.increment_by(bytes_read as u64);
                meter.notify(false);
            }
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Asynchronously get the hash of a file.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
///
/// # Returns
///
/// A Result containing a string. If the file is successfully hashed, the result will be `Ok(String)`.
pub async fn async_get_hash_for_file(path: &Path) -> Result<String, FoundationError> {
    let file = TokioFile::open(path).await?;
    let mut reader = TokioBufReader::new(file);
    let mut hasher = Hasher::new();
    tokio::io::copy(&mut reader, &mut hasher).await?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// Asynchronously get the hash of a file with a progress meter.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containg the hash of the file contents in a String or a FoundationError if an
/// error occurs.
pub async fn async_get_hash_for_file_with_meter(
    path: &Path,
    meter: Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError> {
    let mut file = TokioFile::open(path).await?;
    let metadata = file.metadata().await?;
    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut hasher = Hasher::new();

    let mut left_to_read = metadata.len();
    while left_to_read > 0 {
        let bytes_to_read = std::cmp::min(CHUNK_SIZE as u64, left_to_read) as usize;
        let bytes_read = file.read(&mut chunk[..bytes_to_read]).await?;
        left_to_read -= bytes_read as u64;
        hasher.update(&chunk[..bytes_read]);
        if let Ok(mut meter) = meter.lock() {
            meter.increment_by(bytes_read as u64);
            meter.notify(false);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Asynchronously get the hash of a set number of bytes from a file.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `size` - The number of bytes to read from the file.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the file contents in a String or a FoundationError if an
/// error occurs.
pub async fn get_hash_for_file_with_meter_of_bytes(
    path: &Path,
    size: usize,
    meter: Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError> {
    let mut file = TokioFile::open(path).await?;
    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut hasher = Hasher::new();

    let mut left_to_read = size;
    while left_to_read > 0 {
        let bytes_to_read = std::cmp::min(CHUNK_SIZE, left_to_read);
        let bytes_read = file.read(&mut chunk[..bytes_to_read]).await?;
        left_to_read -= bytes_read;
        hasher.update(&chunk[..bytes_read]);
        if let Ok(mut meter) = meter.lock() {
            meter.increment_by(bytes_read as u64);
            meter.notify(false);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Get the hash of a directory with a progress meter.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `include_file_names` - A boolean indicating whether to include file names in the hash.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the directory contents in a String or a FoundationError if an
/// error occurs.
pub fn get_hash_for_dir(
    path: &Path,
    include_file_names: bool,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError> {
    let mut hasher = Hasher::new();
    for entry in walkdir::WalkDir::new(path)
        .min_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let mut file = StdFile::open(entry.path())?;
            let metadata = file.metadata()?;
            let mut chunk = vec![0u8; CHUNK_SIZE];
            let mut left_to_read = metadata.len();
            while left_to_read > 0 {
                let bytes_to_read = std::cmp::min(CHUNK_SIZE as u64, left_to_read) as usize;
                let bytes_read = file.read(&mut chunk[..bytes_to_read])?;
                left_to_read -= bytes_read as u64;
                hasher.update(&chunk[..bytes_read]);
                if let Some(meter) = &meter {
                    if let Ok(mut meter) = meter.lock() {
                        meter.increment_by(bytes_read as u64);
                        meter.notify(false);
                    }
                }
            }
            if include_file_names {
                // Now add the file path to the hash. This lets us distinguish directories that
                // have identical contents, but the different file names.
                let file_path = entry.path().display().to_string();
                hasher.update(file_path.as_bytes());
            }
        } else if entry.file_type().is_dir() {
            if include_file_names {
                // Now add the directory path to the hash. This lets us distinguish directories that
                // have identical contents, but the different directory names.
                let dir_path = entry.path().display().to_string();
                hasher.update(dir_path.as_bytes());
            }
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Asynchronously get the hash of a directory.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `include_file_names` - A boolean indicating whether to include file names in the hash.
///
/// # Returns
///
/// A Result containing a string. If the directory is successfully hashed, the result will be `Ok(String)`.
pub async fn async_get_hash_for_dir(
    path: &Path,
    include_file_names: bool,
) -> Result<String, FoundationError> {
    let mut hasher = Hasher::new();
    for entry in walkdir::WalkDir::new(path)
        .min_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file = TokioFile::open(entry.path()).await?;
            let mut reader = TokioBufReader::new(file);
            tokio::io::copy(&mut reader, &mut hasher).await?;
            if include_file_names {
                // Now add the file path to the hash. This lets us distinguish directories that
                // have identical contents, but the different file names.
                let file_path = entry.path().display().to_string();
                hasher.update(file_path.as_bytes());
            }
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Asynchronously get the hash of a directory with a progress meter.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
/// * `include_file_names` - A boolean indicating whether to include file names in the hash.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the directory contents in a String or a FoundationError if an
/// error occurs.
pub async fn async_get_hash_for_dir_with_meter(
    path: &Path,
    include_file_names: bool,
    meter: &mut Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError> {
    let mut hasher = Hasher::new();
    for entry in walkdir::WalkDir::new(path)
        .min_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let mut file = TokioFile::open(entry.path()).await?;
            let metadata = file.metadata().await?;
            let mut chunk = vec![0u8; CHUNK_SIZE];
            let mut left_to_read = metadata.len();
            while left_to_read > 0 {
                let bytes_to_read = std::cmp::min(CHUNK_SIZE as u64, left_to_read) as usize;
                let bytes_read = file.read(&mut chunk[..bytes_to_read]).await?;
                left_to_read -= bytes_read as u64;
                hasher.update(&chunk[..bytes_read]);
                if let Ok(mut meter) = meter.lock() {
                    meter.increment_by(bytes_read as u64);
                    meter.notify(false);
                }
            }
            if include_file_names {
                // Now add the file path to the hash. This lets us distinguish directories that
                // have identical contents, but the different file names.
                let file_path = entry.path().display().to_string();
                hasher.update(file_path.as_bytes());
            }
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Get the hash of a string.
///
/// # Arguments
///
/// * `input` - A reference to a string.
///
/// # Returns
///
/// A string containing the hash of the input.
#[allow(dead_code)]
pub fn hash_string(input: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize().to_hex().to_string()
}
