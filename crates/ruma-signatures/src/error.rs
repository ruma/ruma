use ruma_common::{
    canonical_json::{JsonType, RedactionError},
    serde::Base64DecodeError,
    EventId, OwnedEventId, OwnedServerName, RoomVersionId,
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

    /// The signature's ID does not have exactly two components separated by a colon.
    #[error("malformed signature ID: expected exactly 2 segment separated by a colon, found {0}")]
    InvalidLength(usize),

    /// The signature's ID contains invalid characters in its version.
    #[error("malformed signature ID: expected version to contain only characters in the character set `[a-zA-Z0-9_]`, found `{0}`")]
    InvalidVersion(String),

    /// PDU was too large
    #[error("PDU is larger than maximum of 65535 bytes")]
    PduSize,
}

impl From<RedactionError> for Error {
    fn from(err: RedactionError) -> Self {
        match err {
            RedactionError::NotOfType { field: target, of_type, .. } => {
                JsonError::NotOfType { target, of_type }.into()
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
    /// `target` is not of the correct type `of_type` ([`JsonType`]).
    #[error("Value in {target:?} must be a JSON {of_type:?}")]
    NotOfType {
        /// An arbitrary "target" that doesn't have the required type.
        target: String,
        /// The JSON type the target was expected to be.
        of_type: JsonType,
    },

    /// Like [`JsonError::NotOfType`], only called when the `target` is a multiple;
    /// array, set, etc.
    #[error("Values in {target:?} must be JSON {of_type:?}s")]
    NotMultiplesOfType {
        /// An arbitrary "target" where
        /// each or one of it's elements doesn't have the required type.
        target: String,
        /// The JSON type the element was expected to be.
        of_type: JsonType,
    },

    /// The given required field is missing from a JSON object.
    #[error("JSON object must contain the field {0:?}")]
    JsonFieldMissingFromObject(String),

    /// A key is missing from a JSON object.
    ///
    /// Note that this is different from [`JsonError::JsonFieldMissingFromObject`],
    /// this error talks about an expected identifying key (`"ed25519:abcd"`)
    /// missing from a target, where the key has a specific "type"/name.
    #[error("JSON object {for_target:?} does not have {type_of} key {with_key:?}")]
    JsonKeyMissing {
        /// The target from which the key is missing.
        for_target: String,
        /// The kind of thing the key indicates.
        type_of: String,
        /// The key that is missing.
        with_key: String,
    },

    /// A more generic JSON error from [`serde_json`].
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

// TODO: make macro for this
impl JsonError {
    pub(crate) fn not_of_type<T: Into<String>>(target: T, of_type: JsonType) -> Error {
        Self::NotOfType { target: target.into(), of_type }.into()
    }

    pub(crate) fn not_multiples_of_type<T: Into<String>>(target: T, of_type: JsonType) -> Error {
        Self::NotMultiplesOfType { target: target.into(), of_type }.into()
    }

    pub(crate) fn field_missing_from_object<T: Into<String>>(target: T) -> Error {
        Self::JsonFieldMissingFromObject(target.into()).into()
    }

    pub(crate) fn key_missing<T1: Into<String>, T2: Into<String>, T3: Into<String>>(
        for_target: T1,
        type_of: T2,
        with_key: T3,
    ) -> Error {
        Self::JsonKeyMissing {
            for_target: for_target.into(),
            type_of: type_of.into(),
            with_key: with_key.into(),
        }
        .into()
    }
}

/// Errors relating to verification of events and signatures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VerificationError {
    /// For when a signature cannot be found for a `target`.
    #[error("Could not find signatures for {0:?}")]
    SignatureNotFound(OwnedServerName),

    /// For when a public key cannot be found for a `target`.
    #[error("Could not find public key for {0:?}")]
    PublicKeyNotFound(OwnedServerName),

    /// For when no public key matches the signature given.
    #[error("Not signed with any of the given public keys")]
    UnknownPublicKeysForSignature,

    /// For when [`ed25519_dalek`] cannot verify a signature.
    #[error("Could not verify signature: {0}")]
    Signature(#[source] ed25519_dalek::SignatureError),
}

impl VerificationError {
    pub(crate) fn signature_not_found(target: OwnedServerName) -> Error {
        Self::SignatureNotFound(target).into()
    }

    pub(crate) fn public_key_not_found(target: OwnedServerName) -> Error {
        Self::PublicKeyNotFound(target).into()
    }
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
    #[error("Event Id {0:?} should have a server name for the given room version {1:?}")]
    ServerNameFromEventIdByRoomVersion(OwnedEventId, RoomVersionId),

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
    pub(crate) fn from_event_id_by_room_version(
        event_id: &EventId,
        room_version: &RoomVersionId,
    ) -> Error {
        Self::ServerNameFromEventIdByRoomVersion(event_id.to_owned(), room_version.clone()).into()
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
