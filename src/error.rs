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
