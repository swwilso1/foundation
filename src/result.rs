//! The `result` module provides the `DynResult` type, a type alias for a `Result` with
//! errors that have the `Send`, `Sync`, and `'static` bounds.

pub type DynResultError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type DynResult<T> = Result<T, DynResultError>;
