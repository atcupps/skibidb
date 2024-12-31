pub mod datatypes;
pub mod storage;

mod error;
pub use error::DBError;
pub use error::DBResult;

#[macro_use]
mod gen_helpers;
