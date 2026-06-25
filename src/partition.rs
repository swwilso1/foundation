//! The `partition` module contains the `PartitionTable` enum which represent partition
//! table types of a disk.

use crate::error::FoundationError;
use crate::filesystem::FileSystem;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// The `PartitionTable` enum represents the different types of partition tables that a disk can
/// have.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartitionTable {
    /// The GPT partition table.
    GPT,

    /// The DOS partition table.
    DOS,
}

impl FromStr for PartitionTable {
    type Err = FoundationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt" => Ok(PartitionTable::GPT),
            "dos" => Ok(PartitionTable::DOS),
            _ => Err(FoundationError::UnknownPartitionTable(s.to_string())),
        }
    }
}

impl fmt::Display for PartitionTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            PartitionTable::DOS => "dos",
            PartitionTable::GPT => "gpt",
        };

        write!(f, "{}", text)
    }
}

impl TryFrom<i64> for PartitionTable {
    type Error = FoundationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PartitionTable::GPT),
            1 => Ok(PartitionTable::DOS),
            _ => Err(FoundationError::InvalidConversion(
                "i64".to_string(),
                "PartitionTable",
            )),
        }
    }
}

impl TryFrom<FileSystem> for PartitionTable {
    type Error = FoundationError;

    fn try_from(value: FileSystem) -> Result<Self, Self::Error> {
        match value {
            FileSystem::Fat16 | FileSystem::Fat32 | FileSystem::ExFat => Ok(PartitionTable::DOS),
            FileSystem::Ext2
            | FileSystem::Ext3
            | FileSystem::Ext4
            | FileSystem::NTFS
            | FileSystem::HFSPlus
            | FileSystem::APFS => Ok(PartitionTable::GPT),
            _ => Err(FoundationError::InvalidConversion(
                "FileSystem".to_string(),
                "PartitionTable",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_partition_table_from_str() {
        let gpt = PartitionTable::from_str("gpt").unwrap();
        assert_eq!(gpt, PartitionTable::GPT);

        let dos = PartitionTable::from_str("dos").unwrap();
        assert_eq!(dos, PartitionTable::DOS);

        let unknown = PartitionTable::from_str("unknown");
        assert!(unknown.is_err());
    }

    #[test]
    fn test_partition_table_try_from_i64() {
        let gpt = PartitionTable::try_from(0).unwrap();
        assert_eq!(gpt, PartitionTable::GPT);

        let dos = PartitionTable::try_from(1).unwrap();
        assert_eq!(dos, PartitionTable::DOS);

        let unknown = PartitionTable::try_from(2);
        assert!(unknown.is_err());
    }

    #[test]
    fn test_partition_table_display() {
        let gpt = PartitionTable::GPT;
        assert_eq!(gpt.to_string(), "gpt");

        let dos = PartitionTable::DOS;
        assert_eq!(dos.to_string(), "dos");
    }

    #[test]
    fn test_partition_table_try_from_filesystem() {
        // Every DOS-mapped filesystem.
        for fs in [FileSystem::Fat16, FileSystem::Fat32, FileSystem::ExFat] {
            assert_eq!(PartitionTable::try_from(fs).unwrap(), PartitionTable::DOS);
        }

        // Every GPT-mapped filesystem.
        for fs in [
            FileSystem::Ext2,
            FileSystem::Ext3,
            FileSystem::Ext4,
            FileSystem::NTFS,
            FileSystem::HFSPlus,
            FileSystem::APFS,
        ] {
            assert_eq!(PartitionTable::try_from(fs).unwrap(), PartitionTable::GPT);
        }

        // Filesystems that have no partition table mapping.
        for fs in [FileSystem::ISO9660, FileSystem::CIFS] {
            assert!(PartitionTable::try_from(fs).is_err());
        }
    }

    #[test]
    fn test_partition_table_display_from_str_round_trip() {
        for table in [PartitionTable::GPT, PartitionTable::DOS] {
            let rendered = table.to_string();
            assert_eq!(PartitionTable::from_str(&rendered).unwrap(), table);
        }
    }

    #[test]
    fn test_partition_table_try_from_i64_round_trip() {
        // The numeric discriminants used by `TryFrom<i64>`.
        assert_eq!(PartitionTable::try_from(0_i64).unwrap(), PartitionTable::GPT);
        assert_eq!(PartitionTable::try_from(1_i64).unwrap(), PartitionTable::DOS);

        // Out-of-range values on both ends should fail.
        assert!(PartitionTable::try_from(-1_i64).is_err());
        assert!(PartitionTable::try_from(i64::MAX).is_err());
    }

    #[test]
    fn test_partition_table_from_str_error_payload() {
        // The error should carry the unrecognized input string.
        match PartitionTable::from_str("xfs") {
            Err(FoundationError::UnknownPartitionTable(s)) => assert_eq!(s, "xfs"),
            other => panic!("expected UnknownPartitionTable error, got {:?}", other),
        }
    }
}
