//! Storage related errors that occur when dealing with files and tuples.

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StorageError {
    #[error("I/O error: {0}")]
    IOError(#[from] io::Error),

    #[error("invalid function argument: {0}")]
    InvalidArgument(String),

    #[error("could not find page with page ID: {0}")]
    UnknownPage(u64),

    #[error("buffer pool is full ({0} pages in pool) and no page can be evicted")]
    BufferPoolFull(u64),

    #[error("cannot delete file while pages are still pinned")]
    DeleteFileWhilePagesPinned,
}
