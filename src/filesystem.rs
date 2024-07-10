//! The `filesystems` module contains the `FileSystem` enum, which represent
//! the different types of file systems a partition can have.

use crate::error::FoundationError;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// The `FileSystem` enum represents the different types of filesystems that a partition can have.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileSystem {
    /// The ext2 filesystem.
    Ext2,

    /// The ext3 filesystem.
    Ext3,

    /// The ext4 filesystem.
    Ext4,

    /// The FAT16 filesystem.
    Fat16,

    /// The FAT32 filesystem.
    Fat32,

    /// The ExFat filesystem.
    ExFat,

    /// The NTFS filesystem.
    NTFS,

    /// The HFS+ filesystem.
    HFSPlus,

    /// The APFS filesystem.
    APFS,

    /// The ISO9660 filesystem.
    ISO9660,

    /// The CIFS filesystem. The CIFS filesystem is commonly for SMB network filesystem shares
    /// and as such isn't really relevant for disk partitions. We might remove this variant
    /// in the future.
    CIFS,
}

// Provide a conversion from a string to a FileSystem enum.
impl FromStr for FileSystem {
    type Err = FoundationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ext2" => Ok(FileSystem::Ext2),
            "ext3" => Ok(FileSystem::Ext3),
            "ext4" => Ok(FileSystem::Ext4),
            "fat16" => Ok(FileSystem::Fat16),
            "vfat" | "fat32" => Ok(FileSystem::Fat32),
            "exfat" => Ok(FileSystem::ExFat),
            "ntfs" => Ok(FileSystem::NTFS),
            "hfsplus" => Ok(FileSystem::HFSPlus),
            "apfs" => Ok(FileSystem::APFS),
            "iso9660" => Ok(FileSystem::ISO9660),
            "cifs" => Ok(FileSystem::CIFS),
            _ => Err(FoundationError::UnknownFilesystem(s.to_string())),
        }
    }
}

// Provide a conversion from a FileSystem enum to a string.
impl fmt::Display for FileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            FileSystem::Ext2 => "ext2",
            FileSystem::Ext3 => "ext3",
            FileSystem::Ext4 => "ext4",
            FileSystem::Fat16 | FileSystem::Fat32 => "vfat",
            FileSystem::ExFat => "exfat",
            FileSystem::NTFS => "ntfs",
            FileSystem::HFSPlus => "hfsplus",
            FileSystem::APFS => "apfs",
            FileSystem::ISO9660 => "iso9660",
            FileSystem::CIFS => "cifs",
        };

        write!(f, "{}", text)
    }
}

// Provide a conversion from an i64 to a FileSystem enum.
impl TryFrom<i64> for FileSystem {
    type Error = FoundationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FileSystem::Ext2),
            1 => Ok(FileSystem::Ext3),
            2 => Ok(FileSystem::Ext4),
            3 => Ok(FileSystem::Fat16),
            4 => Ok(FileSystem::Fat32),
            5 => Ok(FileSystem::ExFat),
            6 => Ok(FileSystem::NTFS),
            7 => Ok(FileSystem::HFSPlus),
            8 => Ok(FileSystem::APFS),
            9 => Ok(FileSystem::ISO9660),
            10 => Ok(FileSystem::CIFS),
            _ => Err(FoundationError::InvalidConversion(
                "i64".to_string(),
                "FileSystem",
            )),
        }
    }
}

/// Determine if a filesystem is mountable.
///
/// # Arguments
///
/// * `fs` - The filesystem to check.
///
/// # Returns
///
/// A boolean indicating if the filesystem is mountable.
pub fn filesystem_is_mountable(fs: FileSystem) -> bool {
    match fs {
        FileSystem::Ext2
        | FileSystem::Ext3
        | FileSystem::Ext4
        | FileSystem::Fat16
        | FileSystem::Fat32
        | FileSystem::ExFat
        | FileSystem::ISO9660
        | FileSystem::CIFS => true,
        FileSystem::NTFS | FileSystem::HFSPlus | FileSystem::APFS => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_from_str() {
        assert_eq!(FileSystem::from_str("ext2").unwrap(), FileSystem::Ext2);
        assert_eq!(FileSystem::from_str("ext3").unwrap(), FileSystem::Ext3);
        assert_eq!(FileSystem::from_str("ext4").unwrap(), FileSystem::Ext4);
        assert_eq!(FileSystem::from_str("fat16").unwrap(), FileSystem::Fat16);
        assert_eq!(FileSystem::from_str("vfat").unwrap(), FileSystem::Fat32);
        assert_eq!(FileSystem::from_str("fat32").unwrap(), FileSystem::Fat32);
        assert_eq!(FileSystem::from_str("exfat").unwrap(), FileSystem::ExFat);
        assert_eq!(FileSystem::from_str("ntfs").unwrap(), FileSystem::NTFS);
        assert_eq!(
            FileSystem::from_str("hfsplus").unwrap(),
            FileSystem::HFSPlus
        );
        assert_eq!(FileSystem::from_str("apfs").unwrap(), FileSystem::APFS);
        assert_eq!(
            FileSystem::from_str("iso9660").unwrap(),
            FileSystem::ISO9660
        );
        assert_eq!(FileSystem::from_str("cifs").unwrap(), FileSystem::CIFS);
        assert!(FileSystem::from_str("invalid").is_err());
    }

    #[test]
    fn test_filesystem_display() {
        assert_eq!(format!("{}", FileSystem::Ext2), "ext2");
        assert_eq!(format!("{}", FileSystem::Ext3), "ext3");
        assert_eq!(format!("{}", FileSystem::Ext4), "ext4");
        assert_eq!(format!("{}", FileSystem::Fat16), "vfat");
        assert_eq!(format!("{}", FileSystem::Fat32), "vfat");
        assert_eq!(format!("{}", FileSystem::ExFat), "exfat");
        assert_eq!(format!("{}", FileSystem::NTFS), "ntfs");
        assert_eq!(format!("{}", FileSystem::HFSPlus), "hfsplus");
        assert_eq!(format!("{}", FileSystem::APFS), "apfs");
        assert_eq!(format!("{}", FileSystem::ISO9660), "iso9660");
        assert_eq!(format!("{}", FileSystem::CIFS), "cifs");
    }

    #[test]
    fn test_filesystem_try_from() {
        assert_eq!(FileSystem::try_from(0).unwrap(), FileSystem::Ext2);
        assert_eq!(FileSystem::try_from(1).unwrap(), FileSystem::Ext3);
        assert_eq!(FileSystem::try_from(2).unwrap(), FileSystem::Ext4);
        assert_eq!(FileSystem::try_from(3).unwrap(), FileSystem::Fat16);
        assert_eq!(FileSystem::try_from(4).unwrap(), FileSystem::Fat32);
        assert_eq!(FileSystem::try_from(5).unwrap(), FileSystem::ExFat);
        assert_eq!(FileSystem::try_from(6).unwrap(), FileSystem::NTFS);
        assert_eq!(FileSystem::try_from(7).unwrap(), FileSystem::HFSPlus);
        assert_eq!(FileSystem::try_from(8).unwrap(), FileSystem::APFS);
        assert_eq!(FileSystem::try_from(9).unwrap(), FileSystem::ISO9660);
        assert_eq!(FileSystem::try_from(10).unwrap(), FileSystem::CIFS);
        assert!(FileSystem::try_from(11).is_err());
    }
}
