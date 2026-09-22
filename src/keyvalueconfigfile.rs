//! The `keyvalueconfigfile` module provides the `KeyValueConfigFile` object used to read and
//! write configuration files that have a simple key = value format.
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use std::env::temp_dir;
//! use std::path::PathBuf;
//! use foundation::keyvalueconfigfile::KeyValueConfigFile;
//!
//! fn main() {
//!    let mut temp_path = temp_dir();
//!    temp_path.push("configuration.txt");
//!    let config_file = KeyValueConfigFile::new(temp_path);
//!    let mut configuration = HashMap::new();
//!    configuration.insert("key1".to_string(), "value1".to_string());
//!    configuration.insert("key2".to_string(), "value2".to_string());
//!
//!   // Save the configuration to the file.
//!   config_file.save_configuration(&configuration).unwrap();
//!
//!   // Load the configuration from the file.
//!   let loaded_configuration = config_file.load_configuration().unwrap();
//!
//!   // The loaded configuration should be the same as the original configuration.
//!   assert_eq!(configuration, loaded_configuration);
//! }
//! ```

use crate::error::FoundationError;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// The `KeyValueConfigFile` object is used to read and write configuration files that have a simple
/// key = value format.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyValueConfigFile {
    /// The path to the configuration file.
    filename: PathBuf,
}

impl KeyValueConfigFile {
    /// Create a new `KeyValueConfigFile` object.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file.
    pub fn new(path: PathBuf) -> KeyValueConfigFile {
        KeyValueConfigFile { filename: path }
    }

    /// Load the configuration from the file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` of the configuration key value pairs if the file was
    /// successfully read, otherwise a `FoundationError` is returned.
    pub fn load_configuration(&self) -> Result<HashMap<String, String>, FoundationError> {
        match std::fs::read_to_string(&self.filename) {
            Ok(contents) => {
                let mut configuration = HashMap::new();
                for line in contents.lines() {
                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Skip lines that are comments.
                    if line.chars().nth(0).unwrap() == '#' {
                        continue;
                    }

                    // Trim off a newline character if it exists.
                    let the_line = if line.ends_with('\n') {
                        &line[0..line.len() - 1]
                    } else {
                        line
                    };

                    let parts: Vec<&str> = the_line.splitn(2, '=').collect();

                    // Only use lines that have a key = value, otherwise discard them.
                    if parts.len() == 2 {
                        configuration.insert(parts[0].to_string(), parts[1].to_string());
                    }
                }
                Ok(configuration)
            }
            Err(e) => Err(FoundationError::IO(e)),
        }
    }

    /// Save the configuration to the file.
    ///
    /// Keys are written in sorted order so that the file contents are deterministic.  Use
    /// `save_ordered_configuration` when the file must be written in a specific order.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The configuration to save to the file.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the configuration was successfully saved to the file, otherwise
    /// a `FoundationError` is returned.
    pub fn save_configuration(
        &self,
        configuration: &HashMap<String, String>,
    ) -> Result<(), FoundationError> {
        let mut entries: Vec<(&str, Option<&str>)> = configuration
            .iter()
            .map(|(key, value)| (key.as_str(), Some(value.as_str())))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        self.save_ordered_configuration(&entries)
    }

    /// Save the configuration entries to the file in the order given.
    ///
    /// # Arguments
    ///
    /// * `entries` - The key value pairs to save.  An entry with a `Some` value is written as
    ///   `key=value`.  An entry with a `None` value is written as a bare `key` line (a flag).
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the configuration was successfully saved to the file, otherwise
    /// a `FoundationError` is returned.
    pub fn save_ordered_configuration<K: AsRef<str>, V: AsRef<str>>(
        &self,
        entries: &[(K, Option<V>)],
    ) -> Result<(), FoundationError> {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.filename)
        {
            Ok(mut file) => {
                for (key, value) in entries {
                    match value {
                        Some(value) => writeln!(file, "{}={}", key.as_ref(), value.as_ref())?,
                        None => writeln!(file, "{}", key.as_ref())?,
                    }
                }
                Ok(())
            }
            Err(e) => Err(FoundationError::IO(e)),
        }
    }

    /// Check if the file exists.
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    pub fn file_exists(&self) -> bool {
        self.filename.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use tempfile::tempdir;

    #[test]
    fn test_load_configuration() {
        let mut temp_path = temp_dir();
        temp_path.push("keyvalueconfigfile_test_load_configuration.txt");
        let file = KeyValueConfigFile::new(temp_path);
        let mut configuration = HashMap::new();
        configuration.insert("key1".to_string(), "value1".to_string());
        configuration.insert("key2".to_string(), "value2".to_string());
        configuration.insert("key3".to_string(), "value3".to_string());
        configuration.insert("key4".to_string(), "value4".to_string());
        configuration.insert("key5".to_string(), "value5".to_string());
        configuration.insert("key6".to_string(), "value6".to_string());
        configuration.insert("key7".to_string(), "value7".to_string());
        configuration.insert("key8".to_string(), "value8".to_string());
        configuration.insert("key9".to_string(), "value9".to_string());
        configuration.insert("key10".to_string(), "value10".to_string());
        configuration.insert("key11".to_string(), "value11".to_string());
        configuration.insert("key12".to_string(), "value12".to_string());
        configuration.insert("key13".to_string(), "value13".to_string());
        configuration.insert("key14".to_string(), "value14".to_string());
        configuration.insert("key15".to_string(), "value15".to_string());
        configuration.insert("key16".to_string(), "value16".to_string());
        configuration.insert("key17".to_string(), "value17".to_string());
        configuration.insert("key18".to_string(), "value18".to_string());
        configuration.insert("key19".to_string(), "value19".to_string());
        configuration.insert("key20".to_string(), "value20".to_string());
        file.save_configuration(&configuration).unwrap();
        let loaded_configuration = file.load_configuration().unwrap();
        assert_eq!(configuration, loaded_configuration);
        assert!(file.file_exists());
    }

    #[test]
    fn test_new_stores_filename() {
        let path = PathBuf::from("/some/path/config.txt");
        let file = KeyValueConfigFile::new(path.clone());
        assert_eq!(file.filename, path);
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let dir = tempdir().unwrap();
        let file = KeyValueConfigFile::new(dir.path().join("config.txt"));
        let mut configuration = HashMap::new();
        configuration.insert("alpha".to_string(), "one".to_string());
        configuration.insert("beta".to_string(), "two".to_string());

        file.save_configuration(&configuration).unwrap();
        let loaded = file.load_configuration().unwrap();
        assert_eq!(configuration, loaded);
    }

    #[test]
    fn test_load_nonexistent_file_returns_io_error() {
        let dir = tempdir().unwrap();
        let file = KeyValueConfigFile::new(dir.path().join("does_not_exist.txt"));
        let result = file.load_configuration();
        assert!(matches!(result, Err(FoundationError::IO(_))));
    }

    #[test]
    fn test_load_skips_comments_and_empty_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        std::fs::write(
            &path,
            "# This is a comment\n\nkey1=value1\n# another comment\nkey2=value2\n\n",
        )
        .unwrap();
        let file = KeyValueConfigFile::new(path);

        let loaded = file.load_configuration().unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), "value1".to_string());
        expected.insert("key2".to_string(), "value2".to_string());
        assert_eq!(loaded, expected);
    }

    #[test]
    fn test_load_discards_lines_without_equals() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        std::fs::write(&path, "key1=value1\nnoequalsline\nkey2=value2\n").unwrap();
        let file = KeyValueConfigFile::new(path);

        let loaded = file.load_configuration().unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), "value1".to_string());
        expected.insert("key2".to_string(), "value2".to_string());
        assert_eq!(loaded, expected);
        assert!(!loaded.contains_key("noequalsline"));
    }

    #[test]
    fn test_load_value_containing_equals_is_preserved() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        // Only the first '=' should split the key from the value.
        std::fs::write(&path, "key=a=b=c\n").unwrap();
        let file = KeyValueConfigFile::new(path);

        let loaded = file.load_configuration().unwrap();
        assert_eq!(loaded.get("key"), Some(&"a=b=c".to_string()));
    }

    #[test]
    fn test_load_allows_empty_value() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        std::fs::write(&path, "key=\n").unwrap();
        let file = KeyValueConfigFile::new(path);

        let loaded = file.load_configuration().unwrap();
        assert_eq!(loaded.get("key"), Some(&"".to_string()));
    }

    #[test]
    fn test_save_writes_empty_value_with_equals() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path.clone());
        let mut configuration = HashMap::new();
        configuration.insert("flag".to_string(), "".to_string());

        file.save_configuration(&configuration).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        // A key with an empty value is written as "key=" so it round-trips.
        assert_eq!(contents, "flag=\n");

        // The empty-valued key survives a save -> load round trip.
        let loaded = file.load_configuration().unwrap();
        assert_eq!(loaded, configuration);
    }

    #[test]
    fn test_save_writes_keys_in_sorted_order() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path.clone());
        let mut configuration = HashMap::new();
        configuration.insert("gamma".to_string(), "3".to_string());
        configuration.insert("alpha".to_string(), "1".to_string());
        configuration.insert("beta".to_string(), "2".to_string());

        file.save_configuration(&configuration).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "alpha=1\nbeta=2\ngamma=3\n");
    }

    #[test]
    fn test_save_ordered_preserves_order_and_writes_bare_flags() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path.clone());
        let entries = [
            ("zeta", Some("last-alphabetically")),
            ("flag", None),
            ("alpha", Some("")),
        ];

        file.save_ordered_configuration(&entries).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        // Order is preserved, None is a bare key, and an empty Some value keeps its "=".
        assert_eq!(contents, "zeta=last-alphabetically\nflag\nalpha=\n");
    }

    #[test]
    fn test_save_truncates_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path.clone());

        let mut first = HashMap::new();
        first.insert("old_key".to_string(), "old_value".to_string());
        file.save_configuration(&first).unwrap();

        let mut second = HashMap::new();
        second.insert("new_key".to_string(), "new_value".to_string());
        file.save_configuration(&second).unwrap();

        let loaded = file.load_configuration().unwrap();
        assert_eq!(loaded, second);
        assert!(!loaded.contains_key("old_key"));
    }

    #[test]
    fn test_save_empty_configuration_creates_empty_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path.clone());

        file.save_configuration(&HashMap::new()).unwrap();
        assert!(file.file_exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "");
        assert!(file.load_configuration().unwrap().is_empty());
    }

    #[test]
    fn test_file_exists_false_then_true() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.txt");
        let file = KeyValueConfigFile::new(path);
        assert!(!file.file_exists());

        file.save_configuration(&HashMap::new()).unwrap();
        assert!(file.file_exists());
    }
}
