//! The `hash` module provides synchronous and asynchronous functions to hash files and directories.p
//! The functions in this module use the `blake3` crate to hash files and directories. The asynchronous
//! functions use the `tokio` crate to perform the asynchronous operations.

use crate::error::FoundationError;
use blake3::Hasher;
use std::fs::File as StdFile;
use std::io::BufReader as StdBufReader;
use std::path::Path;
use tokio::{fs::File as TokioFile, io::BufReader as TokioBufReader};

/// Get the hash of a file.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
///
/// # Returns
///
/// A Result containing a string. If the file is successfully hashed, the result will be `Ok(String)`.
pub fn get_hash_for_file(path: &Path) -> Result<String, FoundationError> {
    let file = StdFile::open(path)?;
    let mut reader = StdBufReader::new(file);
    let mut hasher = Hasher::new();
    std::io::copy(&mut reader, &mut hasher)?;
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

/// Get the hash of a directory.
///
/// # Arguments
///
/// * `path` - A reference to a Path.
///
/// # Returns
///
/// A Result containing a string. If the directory is successfully hashed, the result will be `Ok(String)`.
pub fn get_hash_for_dir(path: &Path, include_file_names: bool) -> Result<String, FoundationError> {
    let mut hasher = Hasher::new();
    for entry in walkdir::WalkDir::new(path)
        .min_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file = StdFile::open(entry.path())?;
            let mut reader = StdBufReader::new(file);
            std::io::copy(&mut reader, &mut hasher)?;
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
