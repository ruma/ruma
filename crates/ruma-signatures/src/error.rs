use ruma_common::{
    IdParseError, canonical_json::CanonicalJsonFieldError, serde::Base64DecodeError,
};
use thiserror::Error;

use crate::Ed25519VerificationError;

/// All errors related to JSON validation/parsing.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JsonError {
    /// The PDU is too large.
    #[error("PDU is larger than maximum of 65535 bytes")]
    PduTooLarge,

    /// A field is missing or invalid.
    #[error(transparent)]
    Field(#[from] CanonicalJsonFieldError),

    /// A more generic JSON error from [`serde_json`].
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

/// Errors relating to verification of signatures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VerificationError {
    /// The JSON to check is invalid.
    #[error("Invalid JSON: {0}")]
    Json(JsonError),

    /// Parsing a base64-encoded signature failed.
    #[error("Could not parse base64-encoded signature at `path`: {source}")]
    InvalidBase64Signature {
        /// The full path to the signature.
        path: String,

        /// The originating error.
        #[source]
        source: Base64DecodeError,
    },

    /// Parsing a Matrix identifier failed.
    #[error("Could not parse {identifier_type}: {source}")]
    ParseIdentifier {
        /// The type of identifier that was parsed.
        identifier_type: &'static str,

        /// The error when parsing the identifier.
        #[source]
        source: IdParseError,
    },

    /// The signature uses an unsupported algorithm.
    #[error("signature uses an unsupported algorithm")]
    UnsupportedAlgorithm,

    /// The signatures for an entity cannot be found in the signatures map.
    #[error("Could not find signatures for entity {0:?}")]
    NoSignaturesForEntity(String),

    /// No signature with a supported algorithm was found for the given entity.
    #[error("Could not find supported signature for entity {0:?}")]
    NoSupportedSignatureForEntity(String),

    /// Error verifying an ed25519 signature.
    #[error(transparent)]
    Ed25519(#[from] Ed25519VerificationError),
}

impl<T> From<T> for VerificationError
where
    T: Into<JsonError>,
{
    fn from(value: T) -> Self {
        Self::Json(value.into())
    }
}
