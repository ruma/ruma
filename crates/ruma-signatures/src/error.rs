use ruma_identifiers::{EventId, RoomVersionId};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),

    #[error("Verification error: {0}")]
    Verification(#[from] VerificationError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("DER Parse error: {0}")]
    DerParse(pkcs8::der::Error),

    #[error("Split error: {0}")]
    SplitError(#[from] SplitError),

    // For leftover errors that cant be converted yet
    #[error("Error: {0:?}")]
    Misc(String),
}

impl Error {
    // /// Creates a new error.
    // ///
    // /// # Parameters
    // ///
    // /// * message: The error message.
    // pub(crate) fn new<T>(message: T) -> Self
    // where
    //     T: Into<String>,
    // {
    //     Self::Misc(message.into())
    // }
}

// impl From<base64::DecodeError> for Error {
//     fn from(error: base64::DecodeError) -> Self {
//         Self::new(error.to_string())
//     }
// }

// impl From<serde_json::Error> for Error {
//     fn from(error: serde_json::Error) -> Self {
//         Self::new(error.to_string())
//     }
// }

// impl From<ruma_serde::CanonicalJsonError> for Error {
//     fn from(error: ruma_serde::CanonicalJsonError) -> Self {
//         Self::new(error.to_string())
//     }
// }

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JsonError {
    #[error("Value in {target:?} must be a JSON {of_type:?}")]
    NotOfType { target: String, of_type: JsonType },

    #[error("Values in {target:?} must be JSON {of_type:?}s")]
    NotMultiplesOfType { target: String, of_type: JsonType },

    #[error("JSON object must contain the field {0:?}")]
    JsonFieldMissingFromObject(String),

    #[error("JSON object {for_target:?} does not have {type_of} key {with_key:?}")]
    JsonKeyMissing { for_target: String, type_of: String, with_key: String },

    #[error("Canonical JSON error: {0}")]
    CanonicalJson(#[from] ruma_serde::CanonicalJsonError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

// TODO: make macro for this
impl JsonError {
    pub fn not_of_type<T: Into<String>>(target: T, of_type: JsonType) -> Error {
        Self::NotOfType { target: target.into(), of_type }.into()
    }

    pub fn not_multiples_of_type<T: Into<String>>(target: T, of_type: JsonType) -> Error {
        Self::NotMultiplesOfType { target: target.into(), of_type }.into()
    }

    pub fn field_missing_from_object<T: Into<String>>(target: T) -> Error {
        Self::JsonFieldMissingFromObject(target.into()).into()
    }

    pub fn key_missing<T1: Into<String>, T2: Into<String>, T3: Into<String>>(
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

#[derive(Debug)]
pub enum JsonType {
    Object,
    String,
    Integer,
    Array,
    Boolean,
    Null,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VerificationError {
    #[error("Could not find signatures for {0:?}")]
    SignatureNotFound(String),

    #[error("Could not find public key for {0:?}")]
    PublicKeyNotFound(String),

    #[error("Event is not signed with any of the given public keys")]
    UnknownPublicKeysForEvent,

    #[error("Could not verify signature: {0}")]
    Signature(#[source] ed25519_dalek::SignatureError),
}

impl VerificationError {
    pub fn signature_not_found<T: Into<String>>(target: T) -> Error {
        Self::SignatureNotFound(target.into()).into()
    }

    pub fn public_key_not_found<T: Into<String>>(target: T) -> Error {
        Self::PublicKeyNotFound(target.into()).into()
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error("Could not parse User ID: {0}")]
    UserId(#[source] ruma_identifiers::Error),
    #[error("Could not parse Event ID: {0}")]
    EventId(#[source] ruma_identifiers::Error),

    #[error("Event Id {0:?} should have a server name for the given room version {1:?}")]
    ServerNameFromEventIdByRoomVersion(EventId, RoomVersionId),

    #[error("PKCS#8 Document public key does not match public key derived from private key; derived: {0:X?} (len {}), parsed: {1:X?} (len {})", .derived_key.len(), .parsed_key.len())]
    DerivedPublicKeyDoesNotMatchParsedKey { derived_key: Vec<u8>, parsed_key: Vec<u8> },

    #[error("Algorithm OID does not match ed25519, expected {expected}, found {found}")]
    Oid { expected: pkcs8::ObjectIdentifier, found: pkcs8::ObjectIdentifier },

    #[error("Could not parse secret key: {0}")]
    SecretKey(#[source] ed25519_dalek::SignatureError),

    #[error("Could not parse public key: {0}")]
    PublicKey(#[source] ed25519_dalek::SignatureError),

    #[error("Could not parse signature: {0}")]
    Signature(#[source] ed25519_dalek::SignatureError),

    #[error("Could not parse {of_type} base64 string {string:?}: {source}")]
    Base64 {
        of_type: String,
        string: String,
        #[source]
        source: base64::DecodeError,
    },
}

impl ParseError {
    pub fn from_event_id_by_room_version(
        event_id: &EventId,
        room_version: &RoomVersionId,
    ) -> Error {
        Self::ServerNameFromEventIdByRoomVersion(
            // FIX: this can be made better
            event_id.to_owned(),
            room_version.to_owned(),
        )
        .into()
    }

    pub fn derived_vs_parsed_mismatch<D: Into<Vec<u8>>, P: Into<Vec<u8>>>(
        derived: D,
        parsed: P,
    ) -> Error {
        Self::DerivedPublicKeyDoesNotMatchParsedKey {
            derived_key: derived.into(),
            parsed_key: parsed.into(),
        }
        .into()
    }

    pub fn base64<T1: Into<String>, T2: Into<String>>(
        of_type: T1,
        string: T2,
        source: base64::DecodeError,
    ) -> Error {
        Self::Base64 { of_type: of_type.into(), string: string.into(), source }.into()
    }
}

/// An error when trying to extract the algorithm and version from a key identifier.
#[derive(Error, Debug)]
pub enum SplitError {
    /// The signature's ID does not have exactly two components separated by a colon.
    #[error("malformed signature ID: expected exactly 2 segment separated by a colon, found {0}")]
    InvalidLength(usize),

    /// The signature's ID contains invalid characters in its version.
    #[error("malformed signature ID: expected version to contain only characters in the character set `[a-zA-Z0-9_]`, found `{0}`")]
    InvalidVersion(String),

    /// The signature uses an unknown algorithm.
    #[error("unknown algorithm: {0}")]
    UnknownAlgorithm(String),
}

#[test]
fn error_mess() {
    let err = Error::from(ParseError::UserId(ruma_identifiers::Error::EmptyRoomVersionId));

    eprintln!("{}", err);
}
