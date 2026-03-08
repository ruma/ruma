use ruma_common::{
    IdParseError,
    canonical_json::{CanonicalJsonType, RedactionError},
    serde::Base64DecodeError,
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

    /// The field at `path` was expected to be of type `expected`, but was received as `found`.
    #[error("invalid type at `{path}`: expected {expected:?}, found {found:?}")]
    InvalidType {
        /// The path of the invalid field.
        path: String,

        /// The type that was expected.
        expected: CanonicalJsonType,

        /// The type that was found.
        found: CanonicalJsonType,
    },

    /// A required field is missing from a JSON object.
    #[error("missing field: `{path}`")]
    MissingField {
        /// The path of the missing field.
        path: String,
    },

    /// A more generic JSON error from [`serde_json`].
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl From<RedactionError> for JsonError {
    fn from(err: RedactionError) -> Self {
        match err {
            RedactionError::InvalidType { path, expected, found } => {
                JsonError::InvalidType { path, expected, found }
            }
            RedactionError::MissingField { path } => JsonError::MissingField { path },
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

/// Errors relating to verification of signatures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VerificationError {
    /// The JSON to check is invalid.
    #[error("Invalid JSON: {0}")]
    Json(#[from] JsonError),

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

    /// The public keys for an entity cannot be found in the public keys map.
    #[error("Could not find public keys for entity {0:?}")]
    NoPublicKeysForEntity(String),

    /// The public key with the given identifier cannot be found for the given entity.
    #[error("Could not find public key {key_id:?} for entity {entity:?}")]
    PublicKeyNotFound {
        /// The entity for which the key is missing.
        entity: String,

        /// The identifier of the key that is missing.
        key_id: String,
    },

    /// No signature with a supported algorithm was found for the given entity.
    #[error("Could not find supported signature for entity {0:?}")]
    NoSupportedSignatureForEntity(String),

    /// Error verifying an ed25519 signature.
    #[error(transparent)]
    Ed25519(#[from] Ed25519VerificationError),
}
