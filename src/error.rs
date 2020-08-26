use std::num::ParseIntError;

use serde_json::Error as JsonError;
use thiserror::Error;

/// Result type for state resolution.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents the various errors that arise when resolving state.
#[derive(Error, Debug)]
pub enum Error {
    /// A deserialization error.
    #[error(transparent)]
    SerdeJson(#[from] JsonError),

    /// An error that occurs when converting from JSON numbers to rust.
    #[error(transparent)]
    IntParseError(#[from] ParseIntError),

    #[error("Not found error: {0}")]
    NotFound(String),

    // TODO remove once the correct errors are used
    #[error("an error occured {0}")]
    TempString(String),
}
