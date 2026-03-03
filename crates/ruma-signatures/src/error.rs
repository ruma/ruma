use ruma_common::{
    OwnedEventId,
    canonical_json::{CanonicalJsonType, RedactionError},
    serde::Base64DecodeError,
};
use thiserror::Error;

use crate::Ed25519VerificationError;

/// `ruma-signature`'s error type, wraps a number of other error types.
#[derive(Debug, Error)]
#[non_exhaustive]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    /// [`JsonError`] wrapper.
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),

    /// [`VerificationError`] wrapper.
    #[error("Verification error: {0}")]
    Verification(#[from] VerificationError),

    /// [`ParseError`] wrapper.
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}

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

/// Errors relating to parsing of all sorts.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// For user ID parsing errors.
    #[error("Could not parse User ID: {0}")]
    UserId(#[source] ruma_common::IdParseError),

    /// For event ID parsing errors.
    #[error("Could not parse Event ID: {0}")]
    EventId(#[source] ruma_common::IdParseError),

    /// For when an event ID, coupled with a specific room version, doesn't have a server name
    /// embedded.
    #[error("Event ID {0:?} should have a server name for the given room version")]
    ServerNameFromEventId(OwnedEventId),

    /// For when parsing base64 gives an error.
    #[error("Could not parse {of_type} base64 string {string:?}: {source}")]
    Base64 {
        /// The "type"/name of the base64 string
        of_type: String,
        /// The string itself.
        string: String,
        /// The originating error.
        #[source]
        source: Base64DecodeError,
    },
}

impl ParseError {
    pub(crate) fn server_name_from_event_id(event_id: OwnedEventId) -> Error {
        Self::ServerNameFromEventId(event_id).into()
    }

    pub(crate) fn base64<T1: Into<String>, T2: Into<String>>(
        of_type: T1,
        string: T2,
        source: Base64DecodeError,
    ) -> Error {
        Self::Base64 { of_type: of_type.into(), string: string.into(), source }.into()
    }
}
