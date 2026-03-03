use ruma_common::{
    OwnedEventId,
    canonical_json::{CanonicalJsonType, RedactionError},
    serde::Base64DecodeError,
};
use thiserror::Error;

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

    /// Wrapper for [`pkcs8::Error`].
    #[error("DER Parse error: {0}")]
    DerParse(pkcs8::Error),

    /// PDU was too large
    #[error("PDU is larger than maximum of 65535 bytes")]
    PduSize,
}

impl From<RedactionError> for Error {
    fn from(err: RedactionError) -> Self {
        match err {
            RedactionError::InvalidType { path, expected, found } => {
                JsonError::InvalidType { path, expected, found }.into()
            }
            RedactionError::JsonFieldMissingFromObject(field) => {
                JsonError::JsonFieldMissingFromObject(field).into()
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

/// All errors related to JSON validation/parsing.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JsonError {
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

    /// The given required field is missing from a JSON object.
    #[error("JSON object must contain the field {0:?}")]
    JsonFieldMissingFromObject(String),

    /// A more generic JSON error from [`serde_json`].
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl JsonError {
    pub(crate) fn field_missing_from_object<T: Into<String>>(target: T) -> Error {
        Self::JsonFieldMissingFromObject(target.into()).into()
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

    /// The signature verification failed.
    #[error("Could not verify signature: {0}")]
    Signature(#[source] ed25519_dalek::SignatureError),
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

    /// For when the extracted/"parsed" public key from a PKCS#8 v2 document doesn't match the
    /// public key derived from it's private key.
    #[error("PKCS#8 Document public key does not match public key derived from private key; derived: {0:X?} (len {}), parsed: {1:X?} (len {})", .derived_key.len(), .parsed_key.len())]
    DerivedPublicKeyDoesNotMatchParsedKey {
        /// The parsed key.
        parsed_key: Vec<u8>,
        /// The derived key.
        derived_key: Vec<u8>,
    },

    /// For when the ASN.1 Object Identifier on a PKCS#8 document doesn't match the expected one.
    ///
    /// e.g. the document describes a RSA key, while an ed25519 key was expected.
    #[error("Algorithm OID does not match ed25519, expected {expected}, found {found}")]
    Oid {
        /// The expected OID.
        expected: pkcs8::ObjectIdentifier,
        /// The OID that was found instead.
        found: pkcs8::ObjectIdentifier,
    },

    /// For when [`ed25519_dalek`] cannot parse a secret/private key.
    #[error("Could not parse secret key")]
    SecretKey,

    /// For when [`ed25519_dalek`] cannot parse a public key.
    #[error("Could not parse public key: {0}")]
    PublicKey(#[source] ed25519_dalek::SignatureError),

    /// For when [`ed25519_dalek`] cannot parse a signature.
    #[error("Could not parse signature: {0}")]
    Signature(#[source] ed25519_dalek::SignatureError),

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

    pub(crate) fn derived_vs_parsed_mismatch<P: Into<Vec<u8>>, D: Into<Vec<u8>>>(
        parsed: P,
        derived: D,
    ) -> Error {
        Self::DerivedPublicKeyDoesNotMatchParsedKey {
            parsed_key: parsed.into(),
            derived_key: derived.into(),
        }
        .into()
    }

    pub(crate) fn base64<T1: Into<String>, T2: Into<String>>(
        of_type: T1,
        string: T2,
        source: Base64DecodeError,
    ) -> Error {
        Self::Base64 { of_type: of_type.into(), string: string.into(), source }.into()
    }
}
