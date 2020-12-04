//! Modules for events in the *m.key.verification* namespace.
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! The MSC for the in-room variants of the `m.key.verification.*` events can be found
//! [here](https://github.com/matrix-org/matrix-doc/pull/2241).

#[cfg(feature = "unstable-pre-spec")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable-pre-spec")]
use std::convert::TryFrom;

#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::EventId;
use ruma_serde::StringEnum;

#[cfg(feature = "unstable-pre-spec")]
use crate::room::relationships::{Reference, RelatesToJsonRepr, RelationJsonRepr};

pub mod accept;
pub mod cancel;
#[cfg(feature = "unstable-pre-spec")]
pub mod done;
pub mod key;
pub mod mac;
#[cfg(feature = "unstable-pre-spec")]
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

/// The relation that contains info which event the reaction is applying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "RelatesToJsonRepr", into = "RelatesToJsonRepr")]
#[cfg(feature = "unstable-pre-spec")]
pub struct Relation {
    /// The event that is being referenced.
    pub event_id: EventId,
}

#[cfg(feature = "unstable-pre-spec")]
impl From<Relation> for RelatesToJsonRepr {
    fn from(relation: Relation) -> Self {
        RelatesToJsonRepr::Relation(RelationJsonRepr::Reference(Reference {
            event_id: relation.event_id,
        }))
    }
}

#[cfg(feature = "unstable-pre-spec")]
impl TryFrom<RelatesToJsonRepr> for Relation {
    type Error = &'static str;

    fn try_from(value: RelatesToJsonRepr) -> Result<Self, Self::Error> {
        if let RelatesToJsonRepr::Relation(RelationJsonRepr::Reference(r)) = value {
            Ok(Relation { event_id: r.event_id })
        } else {
            Err("Expected a relation with a rel_type of `reference`")
        }
    }
}

/// A Short Authentication String (SAS) verification method.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum VerificationMethod {
    /// The *m.sas.v1* verification method.
    #[ruma_enum(rename = "m.sas.v1")]
    MSasV1,

    #[doc(hidden)]
    _Custom(String),
}

#[cfg(test)]
mod test {
    use super::{KeyAgreementProtocol, MessageAuthenticationCode};

    use serde_json::{from_value as from_json_value, json};

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
