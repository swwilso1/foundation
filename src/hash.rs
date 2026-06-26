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
/// * `aborter` - A function that returns true if the function should abort and false otherwise.
/// * `meter` - An optional reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing a string. If the file is successfully hashed, the result will be `Ok(String)`.
pub fn get_hash_for_file<F>(
    path: &Path,
    aborter: Arc<F>,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
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
        if aborter() {
            return Err(FoundationError::AbortError("Operation aborted".to_string()));
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
/// * `aborter` - A function that returns true if the hash should abort and false otherwise.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containg the hash of the file contents in a String or a FoundationError if an
/// error occurs.
pub async fn async_get_hash_for_file_with_meter<F>(
    path: &Path,
    aborter: Arc<F>,
    meter: Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
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
        if aborter() {
            return Err(FoundationError::AbortError("Operation aborted".to_string()));
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
/// * `aborter` - A function that returns true if the hash should abort and false otherwise.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the file contents in a String or a FoundationError if an
/// error occurs.
pub async fn get_hash_for_file_with_meter_of_bytes<F>(
    path: &Path,
    size: usize,
    aborter: Arc<F>,
    meter: Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
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

        if aborter() {
            return Err(FoundationError::AbortError("Operation aborted".to_string()));
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
/// * `aborter` - A function that returns true if the hash operation should abort and false otherwise.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the directory contents in a String or a FoundationError if an
/// error occurs.
pub fn get_hash_for_dir<F>(
    path: &Path,
    include_file_names: bool,
    aborter: Arc<F>,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
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
                if aborter() {
                    return Err(FoundationError::AbortError("Operation aborted".to_string()));
                }
            }
            if include_file_names {
                // Now add the file path to the hash. This lets us distinguish directories that
                // have identical contents, but the different file names.
                let file_path = entry.path().display().to_string();
                hasher.update(file_path.as_bytes());
            }
        } else if entry.file_type().is_dir()
            && include_file_names {
                // Now add the directory path to the hash. This lets us distinguish directories that
                // have identical contents, but the different directory names.
                let dir_path = entry.path().display().to_string();
                hasher.update(dir_path.as_bytes());
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
/// * `aborter` - A function that returns true if the hash operation should abort.
/// * `meter` - A mutable reference to a ProgressMeter.
///
/// # Returns
///
/// A Result containing the hash of the directory contents in a String or a FoundationError if an
/// error occurs.
pub async fn async_get_hash_for_dir_with_meter<F>(
    path: &Path,
    include_file_names: bool,
    aborter: Arc<F>,
    meter: &mut Arc<Mutex<ProgressMeter>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
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

                if aborter() {
                    return Err(FoundationError::AbortError("Operation aborted".to_string()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};
    use tempfile::tempdir;

    /// The well-known BLAKE3 hash of the empty input.
    const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

    /// An aborter that never aborts.
    fn never() -> Arc<impl Fn() -> bool> {
        Arc::new(|| false)
    }

    /// Compute the expected BLAKE3 hash of a byte slice directly.
    fn expected_hash(bytes: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(bytes);
        hasher.finalize().to_hex().to_string()
    }

    /// Create a `ProgressMeter` (wrapped for sharing) along with a shared tracker that records the
    /// highest percentage reported to the notifier.
    fn meter_with_tracker(total: u64) -> (Arc<Mutex<ProgressMeter>>, Arc<AtomicU8>) {
        let tracker = Arc::new(AtomicU8::new(0));
        let tracker_clone = tracker.clone();
        let meter = ProgressMeter::new_with_notifier_and_size(
            Box::new(move |percent| {
                tracker_clone.fetch_max(percent, Ordering::SeqCst);
            }),
            total,
        );
        (Arc::new(Mutex::new(meter)), tracker)
    }

    // ---- hash_string -----------------------------------------------------

    #[test]
    fn test_hash_string_empty() {
        assert_eq!(hash_string(""), EMPTY_HASH);
    }

    #[test]
    fn test_hash_string_matches_hasher_and_is_deterministic() {
        let input = "the quick brown fox";
        assert_eq!(hash_string(input), expected_hash(input.as_bytes()));
        assert_eq!(hash_string(input), hash_string(input));
        assert_ne!(hash_string("a"), hash_string("b"));
    }

    // ---- get_hash_for_file -----------------------------------------------

    #[test]
    fn test_get_hash_for_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let contents = b"hello world";
        std::fs::write(&path, contents).unwrap();

        let hash = get_hash_for_file(&path, never(), None).unwrap();
        assert_eq!(hash, expected_hash(contents));
    }

    #[test]
    fn test_get_hash_for_file_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        std::fs::write(&path, b"").unwrap();

        let hash = get_hash_for_file(&path, never(), None).unwrap();
        assert_eq!(hash, EMPTY_HASH);
    }

    #[test]
    fn test_get_hash_for_file_larger_than_chunk() {
        // Exercise the multi-iteration read loop with a payload spanning several chunks.
        let dir = tempdir().unwrap();
        let path = dir.path().join("big.bin");
        let contents = vec![0xABu8; CHUNK_SIZE * 2 + 1234];
        std::fs::write(&path, &contents).unwrap();

        let hash = get_hash_for_file(&path, never(), None).unwrap();
        assert_eq!(hash, expected_hash(&contents));
    }

    #[test]
    fn test_get_hash_for_file_with_meter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let contents = vec![7u8; 4096];
        std::fs::write(&path, &contents).unwrap();

        let (meter, tracker) = meter_with_tracker(contents.len() as u64);
        let hash = get_hash_for_file(&path, never(), Some(meter)).unwrap();
        assert_eq!(hash, expected_hash(&contents));
        assert_eq!(tracker.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_get_hash_for_file_aborts() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, vec![0u8; 4096]).unwrap();

        let aborter = Arc::new(|| true);
        let result = get_hash_for_file(&path, aborter, None);
        assert!(matches!(result, Err(FoundationError::AbortError(_))));
    }

    #[test]
    fn test_get_hash_for_file_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist.txt");
        assert!(get_hash_for_file(&path, never(), None).is_err());
    }

    // ---- async_get_hash_for_file -----------------------------------------

    #[tokio::test]
    async fn test_async_get_hash_for_file_matches_sync() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let contents = vec![3u8; CHUNK_SIZE + 17];
        std::fs::write(&path, &contents).unwrap();

        let async_hash = async_get_hash_for_file(&path).await.unwrap();
        let sync_hash = get_hash_for_file(&path, never(), None).unwrap();
        assert_eq!(async_hash, expected_hash(&contents));
        assert_eq!(async_hash, sync_hash);
    }

    #[tokio::test]
    async fn test_async_get_hash_for_file_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.txt");
        assert!(async_get_hash_for_file(&path).await.is_err());
    }

    // ---- async_get_hash_for_file_with_meter ------------------------------

    #[tokio::test]
    async fn test_async_get_hash_for_file_with_meter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let contents = vec![9u8; CHUNK_SIZE * 2];
        std::fs::write(&path, &contents).unwrap();

        let (meter, tracker) = meter_with_tracker(contents.len() as u64);
        let hash = async_get_hash_for_file_with_meter(&path, never(), meter)
            .await
            .unwrap();
        assert_eq!(hash, expected_hash(&contents));
        assert_eq!(tracker.load(Ordering::SeqCst), 100);
    }

    #[tokio::test]
    async fn test_async_get_hash_for_file_with_meter_aborts() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, vec![0u8; 4096]).unwrap();

        let (meter, _tracker) = meter_with_tracker(4096);
        let aborter = Arc::new(|| true);
        let result = async_get_hash_for_file_with_meter(&path, aborter, meter).await;
        assert!(matches!(result, Err(FoundationError::AbortError(_))));
    }

    // ---- get_hash_for_file_with_meter_of_bytes ---------------------------

    #[tokio::test]
    async fn test_get_hash_for_file_with_meter_of_bytes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.bin");
        let contents = vec![5u8; CHUNK_SIZE + 500];
        std::fs::write(&path, &contents).unwrap();

        // Hash only the first `size` bytes; the result must match hashing that prefix directly.
        let size = CHUNK_SIZE + 100;
        let (meter, tracker) = meter_with_tracker(size as u64);
        let hash = get_hash_for_file_with_meter_of_bytes(&path, size, never(), meter)
            .await
            .unwrap();
        assert_eq!(hash, expected_hash(&contents[..size]));
        assert_eq!(tracker.load(Ordering::SeqCst), 100);
    }

    #[tokio::test]
    async fn test_get_hash_for_file_with_meter_of_bytes_zero() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.bin");
        std::fs::write(&path, vec![1u8; 100]).unwrap();

        let (meter, _tracker) = meter_with_tracker(1);
        // Requesting zero bytes never enters the loop and yields the empty hash.
        let hash = get_hash_for_file_with_meter_of_bytes(&path, 0, never(), meter)
            .await
            .unwrap();
        assert_eq!(hash, EMPTY_HASH);
    }

    #[tokio::test]
    async fn test_get_hash_for_file_with_meter_of_bytes_aborts() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.bin");
        std::fs::write(&path, vec![1u8; 4096]).unwrap();

        let (meter, _tracker) = meter_with_tracker(4096);
        let aborter = Arc::new(|| true);
        let result = get_hash_for_file_with_meter_of_bytes(&path, 4096, aborter, meter).await;
        assert!(matches!(result, Err(FoundationError::AbortError(_))));
    }

    // ---- directory hashing helpers ---------------------------------------

    /// Build a small directory tree and return its root path (kept alive by the returned TempDir).
    fn build_tree() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("a.txt"), b"alpha").unwrap();
        std::fs::write(root.join("b.txt"), b"beta").unwrap();
        let sub = root.join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("c.txt"), b"gamma").unwrap();
        dir
    }

    // ---- get_hash_for_dir ------------------------------------------------

    #[test]
    fn test_get_hash_for_dir_deterministic() {
        let tree = build_tree();
        let h1 = get_hash_for_dir(tree.path(), false, never(), None).unwrap();
        let h2 = get_hash_for_dir(tree.path(), false, never(), None).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_get_hash_for_dir_include_file_names_changes_hash() {
        let tree = build_tree();
        let without = get_hash_for_dir(tree.path(), false, never(), None).unwrap();
        let with = get_hash_for_dir(tree.path(), true, never(), None).unwrap();
        assert_ne!(without, with);
    }

    #[test]
    fn test_get_hash_for_dir_with_meter() {
        let tree = build_tree();
        // "alpha" + "beta" + "gamma" = 5 + 4 + 5 = 14 bytes of file content.
        let (meter, tracker) = meter_with_tracker(14);
        get_hash_for_dir(tree.path(), false, never(), Some(meter)).unwrap();
        assert_eq!(tracker.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_get_hash_for_dir_aborts() {
        let tree = build_tree();
        let aborter = Arc::new(|| true);
        let result = get_hash_for_dir(tree.path(), false, aborter, None);
        assert!(matches!(result, Err(FoundationError::AbortError(_))));
    }

    // ---- async_get_hash_for_dir ------------------------------------------

    #[tokio::test]
    async fn test_async_get_hash_for_dir_matches_sync() {
        let tree = build_tree();
        // With `include_file_names = false` both implementations hash only file contents in the
        // same sorted order, so they must agree.
        let async_hash = async_get_hash_for_dir(tree.path(), false).await.unwrap();
        let sync_hash = get_hash_for_dir(tree.path(), false, never(), None).unwrap();
        assert_eq!(async_hash, sync_hash);
    }

    #[tokio::test]
    async fn test_async_get_hash_for_dir_include_file_names_changes_hash() {
        let tree = build_tree();
        let without = async_get_hash_for_dir(tree.path(), false).await.unwrap();
        let with = async_get_hash_for_dir(tree.path(), true).await.unwrap();
        assert_ne!(without, with);
    }

    // ---- async_get_hash_for_dir_with_meter -------------------------------

    #[tokio::test]
    async fn test_async_get_hash_for_dir_with_meter_matches_sync() {
        let tree = build_tree();
        let (mut meter, tracker) = meter_with_tracker(14);
        let hash = async_get_hash_for_dir_with_meter(tree.path(), false, never(), &mut meter)
            .await
            .unwrap();
        let sync_hash = get_hash_for_dir(tree.path(), false, never(), None).unwrap();
        assert_eq!(hash, sync_hash);
        assert_eq!(tracker.load(Ordering::SeqCst), 100);
    }

    #[tokio::test]
    async fn test_async_get_hash_for_dir_with_meter_aborts() {
        let tree = build_tree();
        let (mut meter, _tracker) = meter_with_tracker(14);
        let aborter = Arc::new(|| true);
        let result =
            async_get_hash_for_dir_with_meter(tree.path(), false, aborter, &mut meter).await;
        assert!(matches!(result, Err(FoundationError::AbortError(_))));
    }
}
