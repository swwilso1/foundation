use crate::multiqueue::MultiQueueError;

use notify::Error as NotifyError;
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FoundationError {
    #[error("Could not convert {0} to {1}")]
    InvalidConversion(String, &'static str),

    #[error("Nothing implements {0}")]
    InvalidOperation(String),

    #[error("{0}")]
    OperationFailed(String),

    #[error("IO error: {0}")]
    IO(std::io::Error),

    #[error("Tokio mpsc send error: {0}")]
    TokioMpscSend(String),

    #[error("Unknown files system: {0}")]
    UnknownFilesystem(String),

    #[error("Uknown partition table: {0}")]
    UnknownPartitionTable(String),

    #[error("{0}")]
    GenericError(Box<dyn Error + Send + Sync + 'static>),

    #[error("MultiQueue error: {0}")]
    MultiQueueError(String),

    #[error("Serde YAML error: {0}")]
    SerdeYamlError(serde_yaml::Error),

    #[error("Address Parse error: {0}")]
    AddressParseError(std::net::AddrParseError),

    #[error("Parse integer error: {0}")]
    ParseIntError(std::num::ParseIntError),

    #[error("Unknown Wireless Standard: {0}")]
    UnknownWirelessStandard(String),

    #[error("Unknown Wireless Mode: {0}")]
    UnknownWirelessMode(String),

    #[error("Notify error: {0}")]
    NotifyError(NotifyError),
}

impl From<std::io::Error> for FoundationError {
    fn from(error: std::io::Error) -> Self {
        FoundationError::IO(error)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for FoundationError {
    fn from(value: Box<dyn Error + Send + Sync + 'static>) -> Self {
        FoundationError::GenericError(value)
    }
}

impl<T> From<MultiQueueError<T>> for FoundationError {
    fn from(error: MultiQueueError<T>) -> Self {
        FoundationError::MultiQueueError(error.to_string())
    }
}

impl From<serde_yaml::Error> for FoundationError {
    fn from(error: serde_yaml::Error) -> Self {
        FoundationError::SerdeYamlError(error)
    }
}

impl From<std::net::AddrParseError> for FoundationError {
    fn from(error: std::net::AddrParseError) -> Self {
        FoundationError::AddressParseError(error)
    }
}

impl From<std::num::ParseIntError> for FoundationError {
    fn from(error: std::num::ParseIntError) -> Self {
        FoundationError::ParseIntError(error)
    }
}

impl From<NotifyError> for FoundationError {
    fn from(error: NotifyError) -> Self {
        FoundationError::NotifyError(error)
    }
}
