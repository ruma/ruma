//! Modules for events in the `m.key.verification` namespace.
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! The MSC for the in-room variants of the `m.key.verification.*` events can be found on
//! [MSC2241].
//!
//! [MSC2241]: https://github.com/matrix-org/matrix-spec-proposals/pull/2241

use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, OwnedEventId, PrivOwnedStr};

pub mod accept;
pub mod cancel;
pub mod done;
pub mod key;
pub mod mac;
pub mod ready;
pub mod request;
pub mod start;

/// A hash algorithm.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HashAlgorithm {
    /// The SHA256 hash algorithm.
    Sha256,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl HashAlgorithm {
    /// Creates a string slice from this `HashAlgorithm`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A key agreement protocol.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum KeyAgreementProtocol {
    /// The [Curve25519](https://cr.yp.to/ecdh.html) key agreement protocol.
    Curve25519,

    /// The Curve25519 key agreement protocol with check for public keys.
    Curve25519HkdfSha256,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl KeyAgreementProtocol {
    /// Creates a string slice from this `KeyAgreementProtocol`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A message authentication code algorithm.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MessageAuthenticationCode {
    /// The HKDF-HMAC-SHA256 MAC.
    HkdfHmacSha256,

    /// The HMAC-SHA256 MAC.
    HmacSha256,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl MessageAuthenticationCode {
    /// Creates a string slice from this `MessageAuthenticationCode`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A Short Authentication String method.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ShortAuthenticationString {
    /// The decimal method.
    Decimal,

    /// The emoji method.
    Emoji,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl ShortAuthenticationString {
    /// Creates a string slice from this `ShortAuthenticationString`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A relation which associates an `m.key.verification.request` with another key verification event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.reference")]
pub struct Relation {
    /// The event ID of a related `m.key.verification.request`.
    pub event_id: OwnedEventId,
}

impl Relation {
    /// Creates a new `Relation` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// A Short Authentication String (SAS) verification method.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum VerificationMethod {
    /// The `m.sas.v1` verification method.
    #[ruma_enum(rename = "m.sas.v1")]
    SasV1,

    /// The `m.qr_code.scan.v1` verification method.
    #[ruma_enum(rename = "m.qr_code.scan.v1")]
    QrCodeScanV1,

    /// The `m.qr_code.show.v1` verification method.
    #[ruma_enum(rename = "m.qr_code.show.v1")]
    QrCodeShowV1,

    /// The `m.reciprocate.v1` verification method.
    #[ruma_enum(rename = "m.reciprocate.v1")]
    ReciprocateV1,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl VerificationMethod {
    /// Creates a string slice from this `VerificationMethod`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::{KeyAgreementProtocol, MessageAuthenticationCode};

    #[test]
    fn serialize_key_agreement() {
        let serialized =
            serde_json::to_string(&KeyAgreementProtocol::Curve25519HkdfSha256).unwrap();
        assert_eq!(serialized, "\"curve25519-hkdf-sha256\"");

        let deserialized: KeyAgreementProtocol = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, KeyAgreementProtocol::Curve25519HkdfSha256);
    }

    #[test]
    fn deserialize_mac_method() {
        let json = json!(["hkdf-hmac-sha256", "hmac-sha256"]);

        let deserialized: Vec<MessageAuthenticationCode> = from_json_value(json).unwrap();
        assert!(deserialized.contains(&MessageAuthenticationCode::HkdfHmacSha256));
    }

    #[test]
    fn serialize_mac_method() {
        let serialized = serde_json::to_string(&MessageAuthenticationCode::HkdfHmacSha256).unwrap();
        let deserialized: MessageAuthenticationCode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(serialized, "\"hkdf-hmac-sha256\"");
        assert_eq!(deserialized, MessageAuthenticationCode::HkdfHmacSha256);

        let serialized = serde_json::to_string(&MessageAuthenticationCode::HmacSha256).unwrap();
        let deserialized: MessageAuthenticationCode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(serialized, "\"hmac-sha256\"");
        assert_eq!(deserialized, MessageAuthenticationCode::HmacSha256);
    }
}
