//! The `dir_hasher` module provides code to hash files and directories in a way
//! that prevents multiple hashes of the same file from being computed.

use crate::error::FoundationError;
use crate::hash::get_hash_for_file;
use crate::progressmeter::ProgressMeter;
pub use blake3::Hasher;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A directory entry.
#[derive(Debug)]
pub enum DirEntry {
    File(String, String),

    Dir(String, Box<DirHasher>),
}

/// A directory hasher.
#[derive(Debug)]
pub struct DirHasher {
    hasher: Hasher,
    path: PathBuf,
    children: Vec<DirEntry>,
    hash: Option<String>,
}

impl DirHasher {
    /// Create a new directory hasher.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the directory.
    pub fn new(path: &Path) -> Self {
        DirHasher {
            hasher: Hasher::new(),
            path: path.to_path_buf(),
            children: Vec::new(),
            hash: None,
        }
    }

    /// Compute the hash of the final contents of the directory hasher.
    pub fn hash(&mut self) -> String {
        if let Some(hash) = &self.hash {
            return hash.clone();
        }

        for child in &mut self.children {
            match child {
                DirEntry::File(path, hash) => {
                    self.hasher.update(hash.as_bytes());
                    self.hasher.update(path.as_bytes());
                }
                DirEntry::Dir(_, hasher) => {
                    self.hasher.update(hasher.hash().as_bytes());
                }
            }
        }

        self.hasher
            .update(self.path.display().to_string().as_bytes());
        let hash = self.hasher.finalize().to_hex().to_string();
        self.hash = Some(hash.clone());
        hash
    }

    /// Add a directory entry to the directory hasher.
    ///
    /// # Arguments
    ///
    /// * `entry` - The directory entry to add.
    pub fn add_directory_entry(&mut self, entry: DirEntry) {
        self.children.push(entry);
    }

    /// Return a JSON representation of the contents of the directory represented
    /// by the directory hasher.
    ///
    /// # Returns
    ///
    /// A JSON representation of the directory.
    pub fn get_as_json(&mut self) -> Value {
        let mut children = Vec::new();
        for child in &mut self.children {
            match child {
                DirEntry::File(path, hash) => {
                    children.push(json!({
                        "type": "file",
                        "path": path,
                        "hash": hash,
                    }));
                }
                DirEntry::Dir(_, hasher) => {
                    children.push(hasher.get_as_json());
                }
            }
        }

        json!({
            "path": self.path.display().to_string(),
            "hash": self.hash(),
            "children": children,
            "type": "dir",
        })
    }
}

/// Hash a directory using a DirHasher
///
/// # Arguments
///
/// * `path` - The path to the directory to hash.
/// * `dir_hasher` - The DirHasher to use to hash the directory.
/// * `meter` - An optional progress meter.
///
/// # Returns
///
/// The hash of the directory on success and a FoundationError on failure.
pub fn hash_directory<F>(
    path: &Path,
    dir_hasher: &mut DirHasher,
    aborter: Arc<F>,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
    // `read_dir` yields entries in a filesystem-dependent order, but the directory hash is
    // order-sensitive. Sort the entries so the resulting hash is deterministic regardless of the
    // platform or filesystem the directory lives on.
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in path.read_dir()? {
        entries.push(entry?.path());
    }
    entries.sort();

    for path in entries {
        if path.is_dir() {
            let mut hasher = DirHasher::new(&path);
            hash_directory(&path, &mut hasher, aborter.clone(), meter.clone())?;
            dir_hasher
                .add_directory_entry(DirEntry::Dir(path.display().to_string(), Box::new(hasher)));
        } else {
            hash_file(&path, dir_hasher, aborter.clone(), meter.clone())?;
        }
    }
    Ok(dir_hasher.hash())
}

/// Hash a file and add it to a DirHasher.
///
/// # Arguments
///
/// * `path` - The path to the file to hash.
/// * `dir_hasher` - The DirHasher to add the file to.
/// * `aborter` - A function that returns true if the hash operation should abort.
/// * `meter` - An optional progress meter.
///
/// # Returns
///
/// The hash of the file on success and a FoundationError on failure.
pub fn hash_file<F>(
    path: &Path,
    dir_hasher: &mut DirHasher,
    aborter: Arc<F>,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError>
where
    F: Fn() -> bool,
{
    let hash = get_hash_for_file(path, aborter.clone(), meter)?;
    dir_hasher.add_directory_entry(DirEntry::File(path.display().to_string(), hash.clone()));
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_entry() {
        let dir_entry = DirEntry::File("file".to_string(), "hash".to_string());
        match dir_entry {
            DirEntry::File(path, hash) => {
                assert_eq!(path, "file");
                assert_eq!(hash, "hash");
            }
            _ => panic!("Expected DirEntry::File"),
        }

        let dir_entry = DirEntry::Dir("dir".to_string(), Box::new(DirHasher::new(Path::new(""))));
        match dir_entry {
            DirEntry::Dir(path, _) => {
                assert_eq!(path, "dir");
            }
            _ => panic!("Expected DirEntry::Dir"),
        }
    }

    /// Build the standard nested directory tree used by the hashing tests and return its root.
    ///
    /// The layout is:
    /// ```text
    /// <name>/middle_dir/file1.txt
    ///                  /file2.txt
    ///                  /second_dir/file3.txt
    ///                  /third_dir/file4.txt
    /// ```
    fn build_sample_tree(name: &str) -> PathBuf {
        let start_dir = make_test_dir(name);

        let middle_dir = start_dir.join("middle_dir");
        std::fs::create_dir(&middle_dir).unwrap();

        std::fs::write(middle_dir.join("file1.txt"), "file1").unwrap();
        std::fs::write(middle_dir.join("file2.txt"), "file2").unwrap();

        let second_dir = middle_dir.join("second_dir");
        std::fs::create_dir(&second_dir).unwrap();
        std::fs::write(second_dir.join("file3.txt"), "file3").unwrap();

        let third_dir = middle_dir.join("third_dir");
        std::fs::create_dir(&third_dir).unwrap();
        std::fs::write(third_dir.join("file4.txt"), "file4").unwrap();

        start_dir
    }

    #[test]
    fn test_dir_hasher() {
        // The overall directory hash folds in absolute paths (which differ across platforms and
        // temp-directory locations), so we cannot assert a fixed constant here. Instead verify the
        // properties a directory hash must satisfy: it is a well-formed blake3 hex digest, it is
        // deterministic across independent runs over the same tree, and it changes when the tree's
        // contents change.
        let start_dir = build_sample_tree("test_dir_hasher");

        let mut dir_hasher = DirHasher::new(&start_dir);
        let hash = hash_directory(&start_dir, &mut dir_hasher, Arc::new(|| false), None).unwrap();

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Hashing the same tree again must produce an identical result.
        let mut repeat_hasher = DirHasher::new(&start_dir);
        let repeat =
            hash_directory(&start_dir, &mut repeat_hasher, Arc::new(|| false), None).unwrap();
        assert_eq!(hash, repeat);

        // Changing a file's contents must change the directory hash.
        std::fs::write(start_dir.join("middle_dir").join("file1.txt"), "changed").unwrap();
        let mut changed_hasher = DirHasher::new(&start_dir);
        let changed =
            hash_directory(&start_dir, &mut changed_hasher, Arc::new(|| false), None).unwrap();
        assert_ne!(hash, changed);

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    /// Look up a child object in a JSON children array by its path's file name.
    fn child_by_name<'a>(children: &'a [Value], name: &str) -> &'a Value {
        children
            .iter()
            .find(|child| {
                child
                    .get("path")
                    .and_then(|p| p.as_str())
                    .map(|p| PathBuf::from(p).file_name().unwrap().to_str().unwrap() == name)
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("no child named {}", name))
    }

    #[test]
    fn test_dir_hasher_json() {
        let start_dir = build_sample_tree("test_dir_hasher_json");

        let mut dir_hasher = DirHasher::new(&start_dir);
        let hash = hash_directory(&start_dir, &mut dir_hasher, Arc::new(|| false), None).unwrap();
        let json = dir_hasher.get_as_json();

        // The root reports its own (path-dependent) hash, but that value must match what
        // hash_directory returned, and the structure must reflect the tree we built.
        assert_eq!(json["type"], "dir");
        assert_eq!(json["hash"].as_str().unwrap(), hash);

        let children = json["children"].as_array().unwrap();
        assert_eq!(children.len(), 1);

        let middle = &children[0];
        assert_eq!(middle["type"], "dir");
        let middle_kids = middle["children"].as_array().unwrap();
        assert_eq!(middle_kids.len(), 4);

        // The per-file hashes are content hashes (independent of the absolute path), so they are
        // stable across platforms. Verify each against the standalone file-hash function.
        let middle_dir = start_dir.join("middle_dir");
        let expect_file_hash =
            |path: &Path| -> String { get_hash_for_file(path, Arc::new(|| false), None).unwrap() };

        let file1 = child_by_name(middle_kids, "file1.txt");
        assert_eq!(file1["type"], "file");
        assert_eq!(
            file1["hash"].as_str().unwrap(),
            expect_file_hash(&middle_dir.join("file1.txt"))
        );

        let file2 = child_by_name(middle_kids, "file2.txt");
        assert_eq!(
            file2["hash"].as_str().unwrap(),
            expect_file_hash(&middle_dir.join("file2.txt"))
        );

        let second_dir = child_by_name(middle_kids, "second_dir");
        assert_eq!(second_dir["type"], "dir");
        let second_kids = second_dir["children"].as_array().unwrap();
        assert_eq!(second_kids.len(), 1);
        let file3 = child_by_name(second_kids, "file3.txt");
        assert_eq!(
            file3["hash"].as_str().unwrap(),
            expect_file_hash(&middle_dir.join("second_dir").join("file3.txt"))
        );

        let third_dir = child_by_name(middle_kids, "third_dir");
        assert_eq!(third_dir["type"], "dir");
        let third_kids = third_dir["children"].as_array().unwrap();
        assert_eq!(third_kids.len(), 1);
        let file4 = child_by_name(third_kids, "file4.txt");
        assert_eq!(
            file4["hash"].as_str().unwrap(),
            expect_file_hash(&middle_dir.join("third_dir").join("file4.txt"))
        );

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    /// Create a unique temporary directory for a test, removing any prior copy.
    fn make_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn test_hash_is_cached() {
        // Once hash() has been computed, the value is cached and later mutations to
        // the children do not change the reported hash.
        let mut hasher = DirHasher::new(Path::new("/some/path"));
        hasher.add_directory_entry(DirEntry::File("a".to_string(), "hash_a".to_string()));

        let first = hasher.hash();
        // Calling hash() again returns the same value.
        assert_eq!(first, hasher.hash());

        // Adding a new entry after the hash was cached does not change the result.
        hasher.add_directory_entry(DirEntry::File("b".to_string(), "hash_b".to_string()));
        assert_eq!(first, hasher.hash());
    }

    #[test]
    fn test_hash_is_deterministic_and_path_sensitive() {
        // Two hashers with identical paths and children produce identical hashes.
        let mut a = DirHasher::new(Path::new("/path/one"));
        a.add_directory_entry(DirEntry::File("file".to_string(), "deadbeef".to_string()));

        let mut b = DirHasher::new(Path::new("/path/one"));
        b.add_directory_entry(DirEntry::File("file".to_string(), "deadbeef".to_string()));

        assert_eq!(a.hash(), b.hash());

        // A different path produces a different hash even with identical children.
        let mut c = DirHasher::new(Path::new("/path/two"));
        c.add_directory_entry(DirEntry::File("file".to_string(), "deadbeef".to_string()));

        assert_ne!(a.hash(), c.hash());
    }

    #[test]
    fn test_hash_is_order_sensitive() {
        // The order in which entries are added affects the resulting hash.
        let mut a = DirHasher::new(Path::new("/p"));
        a.add_directory_entry(DirEntry::File("one".to_string(), "h1".to_string()));
        a.add_directory_entry(DirEntry::File("two".to_string(), "h2".to_string()));

        let mut b = DirHasher::new(Path::new("/p"));
        b.add_directory_entry(DirEntry::File("two".to_string(), "h2".to_string()));
        b.add_directory_entry(DirEntry::File("one".to_string(), "h1".to_string()));

        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn test_nested_dir_entry_contributes_to_hash() {
        // A DirEntry::Dir child contributes its own hash to the parent hash.
        let mut child = DirHasher::new(Path::new("/p/child"));
        child.add_directory_entry(DirEntry::File("f".to_string(), "fh".to_string()));

        let mut with_child = DirHasher::new(Path::new("/p"));
        with_child.add_directory_entry(DirEntry::Dir("/p/child".to_string(), Box::new(child)));

        let mut without_child = DirHasher::new(Path::new("/p"));

        assert_ne!(with_child.hash(), without_child.hash());
    }

    #[test]
    fn test_empty_directory() {
        let start_dir = make_test_dir("test_dir_hasher_empty");

        let mut dir_hasher = DirHasher::new(&start_dir);
        let hash = hash_directory(&start_dir, &mut dir_hasher, Arc::new(|| false), None).unwrap();

        // An empty directory hashes the same as a freshly constructed hasher with no
        // children for the same path.
        let mut reference = DirHasher::new(&start_dir);
        assert_eq!(hash, reference.hash());

        // The JSON representation of an empty directory has no children.
        let json = dir_hasher.get_as_json();
        assert_eq!(json["type"], "dir");
        assert_eq!(json["children"].as_array().unwrap().len(), 0);

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    #[test]
    fn test_hash_file_standalone() {
        let start_dir = make_test_dir("test_dir_hasher_hash_file");
        let file = start_dir.join("file.txt");
        std::fs::write(&file, "hello world").unwrap();

        let mut dir_hasher = DirHasher::new(&start_dir);
        let returned = hash_file(&file, &mut dir_hasher, Arc::new(|| false), None).unwrap();

        // The returned hash matches the standalone file hash function.
        let expected = get_hash_for_file(&file, Arc::new(|| false), None).unwrap();
        assert_eq!(returned, expected);

        // The file entry was added to the hasher and is reflected in the JSON.
        let json = dir_hasher.get_as_json();
        let children = json["children"].as_array().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["type"], "file");
        assert_eq!(children[0]["hash"], returned);

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    #[test]
    fn test_hash_directory_with_progress_meter() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let start_dir = make_test_dir("test_dir_hasher_meter");
        let file = start_dir.join("file.txt");
        let contents = "some non-trivial contents for the meter";
        std::fs::write(&file, contents).unwrap();

        let notified = Arc::new(AtomicBool::new(false));
        let notified_clone = notified.clone();
        let meter = ProgressMeter::new_with_notifier_and_size(
            Box::new(move |_percent| {
                notified_clone.store(true, Ordering::SeqCst);
            }),
            contents.len() as u64,
        );
        let meter = Arc::new(Mutex::new(meter));

        let mut dir_hasher = DirHasher::new(&start_dir);
        hash_directory(&start_dir, &mut dir_hasher, Arc::new(|| false), Some(meter)).unwrap();

        // Hashing the file should have driven the progress meter to notify.
        assert!(notified.load(Ordering::SeqCst));

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    #[test]
    fn test_hash_file_aborts() {
        let start_dir = make_test_dir("test_dir_hasher_abort");
        let file = start_dir.join("file.txt");
        std::fs::write(&file, "content that needs reading").unwrap();

        let mut dir_hasher = DirHasher::new(&start_dir);
        let result = hash_file(&file, &mut dir_hasher, Arc::new(|| true), None);

        match result {
            Err(FoundationError::AbortError(_)) => {}
            other => panic!("Expected AbortError, got {:?}", other),
        }

        std::fs::remove_dir_all(&start_dir).unwrap();
    }

    #[test]
    fn test_hash_directory_nonexistent_path() {
        let missing = std::env::temp_dir().join("test_dir_hasher_does_not_exist_xyz");
        if missing.exists() {
            std::fs::remove_dir_all(&missing).unwrap();
        }

        let mut dir_hasher = DirHasher::new(&missing);
        let result = hash_directory(&missing, &mut dir_hasher, Arc::new(|| false), None);
        assert!(result.is_err());
    }
}
