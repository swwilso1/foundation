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

    Dir(String, DirHasher),
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
pub fn hash_directory(
    path: &Path,
    dir_hasher: &mut DirHasher,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError> {
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut hasher = DirHasher::new(&path);
            hash_directory(&path, &mut hasher, meter.clone())?;
            dir_hasher.add_directory_entry(DirEntry::Dir(path.display().to_string(), hasher));
        } else {
            hash_file(&path, dir_hasher, meter.clone())?;
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
/// * `meter` - An optional progress meter.
///
/// # Returns
///
/// The hash of the file on success and a FoundationError on failure.
pub fn hash_file(
    path: &Path,
    dir_hasher: &mut DirHasher,
    meter: Option<Arc<Mutex<ProgressMeter>>>,
) -> Result<String, FoundationError> {
    let hash = get_hash_for_file(path, meter)?;
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

        let dir_entry = DirEntry::Dir("dir".to_string(), DirHasher::new(Path::new("")));
        match dir_entry {
            DirEntry::Dir(path, _) => {
                assert_eq!(path, "dir");
            }
            _ => panic!("Expected DirEntry::Dir"),
        }
    }

    #[test]
    fn test_dir_hasher() {
        let temp_dir = std::env::temp_dir();
        let start_dir = temp_dir.join("test_dir_hasher");

        if start_dir.exists() {
            std::fs::remove_dir_all(&start_dir).unwrap();
        }

        std::fs::create_dir(&start_dir).unwrap();

        let middle_dir = start_dir.join("middle_dir");
        std::fs::create_dir(&middle_dir).unwrap();

        let file1 = middle_dir.join("file1.txt");
        let file2 = middle_dir.join("file2.txt");
        std::fs::write(&file1, "file1").unwrap();
        std::fs::write(&file2, "file2").unwrap();
        let second_dir = middle_dir.join("second_dir");
        std::fs::create_dir(&second_dir).unwrap();
        let file3 = second_dir.join("file3.txt");
        std::fs::write(&file3, "file3").unwrap();

        let third_dir = middle_dir.join("third_dir");
        std::fs::create_dir(&third_dir).unwrap();
        let file4 = third_dir.join("file4.txt");
        std::fs::write(&file4, "file4").unwrap();

        let mut dir_hasher = DirHasher::new(&start_dir);

        let hash = hash_directory(&start_dir, &mut dir_hasher, None).unwrap();
        assert_eq!(
            hash,
            "6fb9784954af75b41e1da47215f98c5e5dd0ea09d0567ce707ff9d42d95bb9fd".to_string()
        );
    }

    #[test]
    fn test_dir_hasher_json() {
        let temp_dir = std::env::temp_dir();
        let start_dir = temp_dir.join("test_dir_hasher_json");

        if start_dir.exists() {
            std::fs::remove_dir_all(&start_dir).unwrap();
        }

        std::fs::create_dir(&start_dir).unwrap();

        let middle_dir = start_dir.join("middle_dir");
        std::fs::create_dir(&middle_dir).unwrap();

        let file1 = middle_dir.join("file1.txt");
        let file2 = middle_dir.join("file2.txt");
        std::fs::write(&file1, "file1").unwrap();
        std::fs::write(&file2, "file2").unwrap();
        let second_dir = middle_dir.join("second_dir");
        std::fs::create_dir(&second_dir).unwrap();
        let file3 = second_dir.join("file3.txt");
        std::fs::write(&file3, "file3").unwrap();

        let third_dir = middle_dir.join("third_dir");
        std::fs::create_dir(&third_dir).unwrap();
        let file4 = third_dir.join("file4.txt");
        std::fs::write(&file4, "file4").unwrap();

        let mut dir_hasher = DirHasher::new(&start_dir);

        let hash = hash_directory(&start_dir, &mut dir_hasher, None).unwrap();
        let json = dir_hasher.get_as_json();

        if let Some(dir_object) = json.as_object() {
            if let Some(hash_object) = dir_object.get("hash") {
                assert_eq!(hash_object.as_str().unwrap(), hash);
            }

            if let Some(children_value) = json.get("children") {
                if let Some(children) = children_value.as_array() {
                    assert_eq!(children.len(), 1);
                    if let Some(child) = children.get(0) {
                        if let Some(child_object) = child.as_object() {
                            if let Some(child_hash) = child_object.get("hash") {
                                assert_eq!(child_hash.as_str().unwrap(), "ed17bcb25c890a296e47385eac148e64e052cb0509a3c6bb34ba2dc578ed7227");
                            }

                            if let Some(middle_kids_value) = child_object.get("children") {
                                if let Some(middle_kids) = middle_kids_value.as_array() {
                                    assert_eq!(middle_kids.len(), 4);
                                    for middle_kid in middle_kids {
                                        if let Some(middle_kid_object) = middle_kid.as_object() {
                                            if let Some(path_value) = middle_kid_object.get("path")
                                            {
                                                if let Some(path_str) = path_value.as_str() {
                                                    let the_path = PathBuf::from(path_str);
                                                    let filename = the_path
                                                        .file_name()
                                                        .unwrap()
                                                        .to_str()
                                                        .unwrap();

                                                    match filename {
                                                        "file1.txt" => {
                                                            if let Some(hash_value) =
                                                                middle_kid_object.get("hash")
                                                            {
                                                                assert_eq!(hash_value.as_str().unwrap(), "ebfeae2a90df171912869c78dc767137bfafbcab6d54ca0fedaca4d2ba824010");
                                                            }
                                                        }
                                                        "file2.txt" => {
                                                            if let Some(hash_value) =
                                                                middle_kid_object.get("hash")
                                                            {
                                                                assert_eq!(hash_value.as_str().unwrap(), "fab1069ff326679acb41b95bf33e7ead4c38ed7a4b994d354b793d759db69ede");
                                                            }
                                                        }
                                                        "second_dir" => {
                                                            if let Some(hash_value) =
                                                                middle_kid_object.get("hash")
                                                            {
                                                                assert_eq!(hash_value.as_str().unwrap(), "a15e4cd5f4ff65b9c8dc3e44400a89a8167fdbf2fca3ee1224813a12f5c86f71");
                                                            }

                                                            if let Some(second_kids_value) =
                                                                middle_kid_object.get("children")
                                                            {
                                                                if let Some(second_kids) =
                                                                    second_kids_value.as_array()
                                                                {
                                                                    assert_eq!(
                                                                        second_kids.len(),
                                                                        1
                                                                    );
                                                                    for second_kid in second_kids {
                                                                        if let Some(
                                                                            second_kid_object,
                                                                        ) =
                                                                            second_kid.as_object()
                                                                        {
                                                                            if let Some(
                                                                                path_value,
                                                                            ) = second_kid_object
                                                                                .get("path")
                                                                            {
                                                                                if let Some(
                                                                                    path_str,
                                                                                ) = path_value
                                                                                    .as_str()
                                                                                {
                                                                                    let the_path = PathBuf::from(path_str);
                                                                                    let filename = the_path.file_name().unwrap().to_str().unwrap();

                                                                                    match filename {
                                                                                        "file3.txt" => {
                                                                                            if let Some(hash_value) = second_kid_object.get("hash") {
                                                                                                assert_eq!(hash_value.as_str().unwrap(), "65e268923166ee125168e80537db6237d64a70b7c4b1b72efe45b3deb25a188d");
                                                                                            }
                                                                                        }
                                                                                        _ => {}
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        "third_dir" => {
                                                            if let Some(hash_value) =
                                                                middle_kid_object.get("hash")
                                                            {
                                                                assert_eq!(hash_value.as_str().unwrap(), "4b2485d1c0d17598da21268e9530decc4b857a5bd8055f79a81b6889d92bd9ca");
                                                            }

                                                            if let Some(third_kids_value) =
                                                                middle_kid_object.get("children")
                                                            {
                                                                if let Some(third_kids) =
                                                                    third_kids_value.as_array()
                                                                {
                                                                    assert_eq!(third_kids.len(), 1);
                                                                    for third_kid in third_kids {
                                                                        if let Some(
                                                                            third_kid_object,
                                                                        ) = third_kid.as_object()
                                                                        {
                                                                            if let Some(
                                                                                path_value,
                                                                            ) = third_kid_object
                                                                                .get("path")
                                                                            {
                                                                                if let Some(
                                                                                    path_str,
                                                                                ) = path_value
                                                                                    .as_str()
                                                                                {
                                                                                    let the_path = PathBuf::from(path_str);
                                                                                    let filename = the_path.file_name().unwrap().to_str().unwrap();

                                                                                    match filename {
                                                                                        "file4.txt" => {
                                                                                            if let Some(hash_value) = third_kid_object.get("hash") {
                                                                                                assert_eq!(hash_value.as_str().unwrap(), "d7a631e53891573df288e8792c751077c54b5a926a55eb94e137f150bafea945");
                                                                                            }
                                                                                        }
                                                                                        _ => {}
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
