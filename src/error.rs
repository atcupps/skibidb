//! Custom errors and related types, functions, impls, and macros for SkibiDB.

use std::io;

use crate::storage::error::StorageError;
use thiserror::Error;

/// A convenience `Result` type that may contain a `DBError`. Generally used
/// for functions, especially publicly-exposed functions, that may return an
/// error within the DBMS.
pub type DBResult<T> = Result<T, DBError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DBError {
    /// Error in the Data Access and Storage System (DASS)
    #[error("DASS error: {0}")]
    StorageError(#[from] StorageError),

    /// I/O error somewhere in the DBMS
    #[error("I/O Error: {0}")]
    IOError(#[from] io::Error),
}
