use serde_json::Error as JsonError;
use thiserror::Error;

/// Result type for state resolution.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents the various errors that arise when resolving state.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// A deserialization error.
    #[error(transparent)]
    SerdeJson(#[from] JsonError),

    /// The given option or version is unsupported.
    #[error("Unsupported room version: {0}")]
    Unsupported(String),

    /// The given event was not found.
    #[error("Not found error: {0}")]
    NotFound(String),

    /// Invalid fields in the given PDU.
    #[error("Invalid PDU: {0}")]
    InvalidPdu(String),

    /// A custom error.
    #[error("{0}")]
    Custom(Box<dyn std::error::Error>),
}

impl Error {
    pub fn custom<E: std::error::Error + 'static>(e: E) -> Self {
        Self::Custom(Box::new(e))
    }
}
