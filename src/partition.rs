//! The `partition` module contains the `PartitionTable` enum which represent partition
//! table types of a disk.

use crate::error::FoundationError;
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
}
