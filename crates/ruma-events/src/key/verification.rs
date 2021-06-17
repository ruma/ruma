//! Modules for events in the *m.key.verification* namespace.
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! The MSC for the in-room variants of the `m.key.verification.*` events can be found
//! [here](https://github.com/matrix-org/matrix-doc/pull/2241).
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::EventId;
use ruma_serde::StringEnum;
#[cfg(feature = "unstable-pre-spec")]
use serde::{Deserialize, Serialize};

pub mod accept;
pub mod cancel;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod done;
pub mod key;
pub mod mac;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod ready;
pub mod request;
pub mod start;

/// A hash algorithm.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum HashAlgorithm {
    /// The SHA256 hash algorithm.
    Sha256,

    #[doc(hidden)]
    _Custom(String),
}

/// A key agreement protocol.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "kebab-case")]
pub enum KeyAgreementProtocol {
    /// The [Curve25519](https://cr.yp.to/ecdh.html) key agreement protocol.
    Curve25519,

    /// The Curve25519 key agreement protocol with check for public keys.
    Curve25519HkdfSha256,

    #[doc(hidden)]
    _Custom(String),
}

/// A message authentication code algorithm.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "kebab-case")]
pub enum MessageAuthenticationCode {
    /// The HKDF-HMAC-SHA256 MAC.
    HkdfHmacSha256,

    /// The HMAC-SHA256 MAC.
    HmacSha256,

    #[doc(hidden)]
    _Custom(String),
}

/// A Short Authentication String method.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum ShortAuthenticationString {
    /// The decimal method.
    Decimal,

    /// The emoji method.
    Emoji,

    #[doc(hidden)]
    _Custom(String),
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type")]
pub enum Relation {
    /// A reference to a related `m.key.verification.request`.
    Reference(VerificationReference),

    /// An unknown relation type.
    ///
    /// Not available in the public API, but exists here so deserialization
    /// doesn't fail with new / custom `rel_type`s.
    #[serde(other)]
    Unknown,
}

/// A reference to a related `m.key.verification.request`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VerificationReference {
    /// The event ID of a related `m.key.verification.request`.
    pub event_id: EventId,
}

#[cfg(feature = "unstable-pre-spec")]
impl VerificationReference {
    /// Creates a new `VerificationReference` with the given event ID.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}

/// A Short Authentication String (SAS) verification method.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum VerificationMethod {
    /// The *m.sas.v1* verification method.
    #[ruma_enum(rename = "m.sas.v1")]
    SasV1,

    /// The *m.qr_code.scan.v1* verification method.
    #[cfg(feature = "unstable-pre-spec")]
    #[ruma_enum(rename = "m.qr_code.scan.v1")]
    QrCodeScanV1,

    /// The *m.qr_code.show.v1* verification method.
    #[cfg(feature = "unstable-pre-spec")]
    #[ruma_enum(rename = "m.qr_code.show.v1")]
    QrCodeShowV1,

    /// The *m.reciprocate.v1* verification method.
    #[cfg(feature = "unstable-pre-spec")]
    #[ruma_enum(rename = "m.reciprocate.v1")]
    ReciprocateV1,

    #[doc(hidden)]
    _Custom(String),
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
