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

    #[error("Invalid PDU: {0}")]
    InvalidPdu(String),

    #[error("Conversion failed: {0}")]
    ConversionError(String),

    #[error("{0}")]
    Custom(Box<dyn std::error::Error>),
}

impl Error {
    pub fn custom<E: std::error::Error + 'static>(e: E) -> Self {
        Self::Custom(Box::new(e))
    }
}
