//! Types for the [`m.key.verification.accept`] event.
//!
//! [`m.key.verification.accept`]: https://spec.matrix.org/v1.18/client-server-api/#mkeyverificationaccept

use std::borrow::Cow;

use as_variant::as_variant;
use ruma_common::{
    OwnedTransactionId,
    serde::{Base64, JsonObject},
};
use ruma_macros::EventContent;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Value as JsonValue, from_value as from_json_value};

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
use crate::relation::Reference;

/// The content of a to-device `m.key.verification.accept` event.
///
/// Accepts a previously sent `m.key.verification.start` message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.key.verification.accept", kind = ToDevice)]
pub struct ToDeviceKeyVerificationAcceptEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the `m.key.verification.start` message.
    pub transaction_id: OwnedTransactionId,

    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,
}

impl ToDeviceKeyVerificationAcceptEventContent {
    /// Creates a new `ToDeviceKeyVerificationAcceptEventContent` with the given transaction ID and
    /// method-specific content.
    pub fn new(transaction_id: OwnedTransactionId, method: AcceptMethod) -> Self {
        Self { transaction_id, method }
    }
}

/// The content of a in-room `m.key.verification.accept` event.
///
/// Accepts a previously sent `m.key.verification.start` message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.key.verification.accept", kind = MessageLike)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct KeyVerificationAcceptEventContent {
    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl KeyVerificationAcceptEventContent {
    /// Creates a new `KeyVerificationAcceptEventContent` with the given method-specific
    /// content and reference.
    pub fn new(method: AcceptMethod, relates_to: Reference) -> Self {
        Self { method, relates_to }
    }
}

/// An enum representing the different method specific `m.key.verification.accept` content.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(untagged)]
pub enum AcceptMethod {
    /// The `m.sas.v1` verification method.
    SasV1(SasV1Content),

    /// Any unknown accept method.
    #[doc(hidden)]
    _Custom(_CustomAcceptMethodContent),
}

impl AcceptMethod {
    /// The value of the `method` field.
    pub fn method(&self) -> &str {
        match self {
            Self::SasV1(_) => "m.sas.v1",
            Self::_Custom(c) => &c.method,
        }
    }

    /// The data of this `AcceptMethod`.
    ///
    /// The returned JSON object won't contain the `method` field, use [`.method()`][Self::method]
    /// to access it.
    ///
    /// Prefer to use the public variants of `AcceptMethod` where possible; this method is meant to
    /// be used for custom methods only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: T) -> JsonObject {
            match serde_json::to_value(obj).expect("accept method serialization to succeed") {
                JsonValue::Object(mut obj) => {
                    obj.remove("method");
                    obj
                }
                _ => panic!("all accept method variants must serialize to objects"),
            }
        }

        match self {
            Self::SasV1(c) => Cow::Owned(serialize(c)),
            Self::_Custom(c) => Cow::Borrowed(&c.data),
        }
    }
}

impl<'de> Deserialize<'de> for AcceptMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut data = JsonObject::deserialize(deserializer)?;

        let method = data
            .get("method")
            .and_then(|value| as_variant!(value, JsonValue::String))
            .ok_or_else(|| de::Error::missing_field("method"))?;

        match method.as_ref() {
            "m.sas.v1" => from_json_value(data.into()).map(Self::SasV1),
            _ => {
                let method = as_variant!(
                    data.remove("method")
                        .expect("we already checked that the method field is present"),
                    JsonValue::String
                )
                .expect("we already checked that the method is a string");

                Ok(Self::_Custom(_CustomAcceptMethodContent { method, data }))
            }
        }
        .map_err(de::Error::custom)
    }
}

/// Method specific content of a unknown key verification method.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
pub struct _CustomAcceptMethodContent {
    /// The name of the method.
    method: String,

    /// The additional fields that the method contains.
    #[serde(flatten)]
    data: JsonObject,
}

/// The payload of an `m.key.verification.accept` event using the `m.sas.v1` method.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(rename = "m.sas.v1", tag = "method")]
pub struct SasV1Content {
    /// The key agreement protocol the device is choosing to use, out of the
    /// options in the `m.key.verification.start` message.
    pub key_agreement_protocol: KeyAgreementProtocol,

    /// The hash method the device is choosing to use, out of the options in the
    /// `m.key.verification.start` message.
    pub hash: HashAlgorithm,

    /// The message authentication code the device is choosing to use, out of
    /// the options in the `m.key.verification.start` message.
    pub message_authentication_code: MessageAuthenticationCode,

    /// The SAS methods both devices involved in the verification process
    /// understand.
    ///
    /// Must be a subset of the options in the `m.key.verification.start`
    /// message.
    pub short_authentication_string: Vec<ShortAuthenticationString>,

    /// The hash (encoded as unpadded base64) of the concatenation of the
    /// device's ephemeral public key (encoded as unpadded base64) and the
    /// canonical JSON representation of the `m.key.verification.start` message.
    pub commitment: Base64,
}

/// Mandatory initial set of fields for creating an accept `SasV1Content`.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SasV1ContentInit {
    /// The key agreement protocol the device is choosing to use, out of the
    /// options in the `m.key.verification.start` message.
    pub key_agreement_protocol: KeyAgreementProtocol,

    /// The hash method the device is choosing to use, out of the options in the
    /// `m.key.verification.start` message.
    pub hash: HashAlgorithm,

    /// The message authentication codes that the accepting device understands.
    pub message_authentication_code: MessageAuthenticationCode,

    /// The SAS methods both devices involved in the verification process
    /// understand.
    ///
    /// Must be a subset of the options in the `m.key.verification.start`
    /// message.
    pub short_authentication_string: Vec<ShortAuthenticationString>,

    /// The hash (encoded as unpadded base64) of the concatenation of the
    /// device's ephemeral public key (encoded as unpadded base64) and the
    /// canonical JSON representation of the `m.key.verification.start` message.
    pub commitment: Base64,
}

impl From<SasV1ContentInit> for SasV1Content {
    /// Creates a new `SasV1Content` from the given init struct.
    fn from(init: SasV1ContentInit) -> Self {
        SasV1Content {
            hash: init.hash,
            key_agreement_protocol: init.key_agreement_protocol,
            message_authentication_code: init.message_authentication_code,
            short_authentication_string: init.short_authentication_string,
            commitment: init.commitment,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::{assert_let, assert_matches};
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq,
        event_id,
        serde::{Base64, Raw},
    };
    use serde_json::{Value as JsonValue, from_value as from_json_value, json};

    use super::{
        AcceptMethod, HashAlgorithm, KeyAgreementProtocol, KeyVerificationAcceptEventContent,
        MessageAuthenticationCode, SasV1Content, ShortAuthenticationString,
        ToDeviceKeyVerificationAcceptEventContent,
    };
    use crate::{ToDeviceEvent, relation::Reference};

    #[test]
    fn to_device_serialization() {
        let key_verification_accept_content = ToDeviceKeyVerificationAcceptEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::SasV1(SasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256V2,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: Base64::new(b"hello".to_vec()),
            }),
        };

        assert_to_canonical_json_eq!(
            key_verification_accept_content,
            json!({
                "transaction_id": "456",
                "method": "m.sas.v1",
                "commitment": "aGVsbG8",
                "key_agreement_protocol": "curve25519",
                "hash": "sha256",
                "message_authentication_code": "hkdf-hmac-sha256.v2",
                "short_authentication_string": ["decimal"],
            }),
        );
    }

    #[test]
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_accept_content = KeyVerificationAcceptEventContent {
            relates_to: Reference { event_id: event_id.to_owned() },
            method: AcceptMethod::SasV1(SasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256V2,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: Base64::new(b"hello".to_vec()),
            }),
        };

        assert_to_canonical_json_eq!(
            key_verification_accept_content,
            json!({
                "method": "m.sas.v1",
                "commitment": "aGVsbG8",
                "key_agreement_protocol": "curve25519",
                "hash": "sha256",
                "message_authentication_code": "hkdf-hmac-sha256.v2",
                "short_authentication_string": ["decimal"],
                "m.relates_to": {
                    "rel_type": "m.reference",
                    "event_id": event_id,
                },
            }),
        );
    }

    #[test]
    fn to_device_deserialization() {
        let json = json!({
            "transaction_id": "456",
            "commitment": "aGVsbG8",
            "method": "m.sas.v1",
            "hash": "sha256",
            "key_agreement_protocol": "curve25519",
            "message_authentication_code": "hkdf-hmac-sha256.v2",
            "short_authentication_string": ["decimal"]
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        let content = from_json_value::<ToDeviceKeyVerificationAcceptEventContent>(json).unwrap();
        assert_eq!(content.transaction_id, "456");

        assert_matches!(content.method, AcceptMethod::SasV1(sas));
        assert_eq!(sas.commitment.encode(), "aGVsbG8");
        assert_eq!(sas.hash, HashAlgorithm::Sha256);
        assert_eq!(sas.key_agreement_protocol, KeyAgreementProtocol::Curve25519);
        assert_eq!(sas.message_authentication_code, MessageAuthenticationCode::HkdfHmacSha256V2);
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);

        let json = json!({
            "content": {
                "commitment": "aGVsbG8",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocol": "curve25519",
                "hash": "sha256",
                "message_authentication_code": "hkdf-hmac-sha256.v2",
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.accept",
            "sender": "@example:localhost",
        });

        let ev = from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationAcceptEventContent>>(json)
            .unwrap();
        assert_eq!(ev.content.transaction_id, "456");
        assert_eq!(ev.sender, "@example:localhost");

        assert_matches!(ev.content.method, AcceptMethod::SasV1(sas));
        assert_eq!(sas.commitment.encode(), "aGVsbG8");
        assert_eq!(sas.hash, HashAlgorithm::Sha256);
        assert_eq!(sas.key_agreement_protocol, KeyAgreementProtocol::Curve25519);
        assert_eq!(sas.message_authentication_code, MessageAuthenticationCode::HkdfHmacSha256V2);
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);
    }

    #[test]
    fn in_room_deserialization() {
        let json = json!({
            "commitment": "aGVsbG8",
            "method": "m.sas.v1",
            "hash": "sha256",
            "key_agreement_protocol": "curve25519",
            "message_authentication_code": "hkdf-hmac-sha256.v2",
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$1598361704261elfgc:localhost",
            }
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        let content = from_json_value::<KeyVerificationAcceptEventContent>(json).unwrap();
        assert_eq!(content.relates_to.event_id, "$1598361704261elfgc:localhost");

        assert_matches!(content.method, AcceptMethod::SasV1(sas));
        assert_eq!(sas.commitment.encode(), "aGVsbG8");
        assert_eq!(sas.hash, HashAlgorithm::Sha256);
        assert_eq!(sas.key_agreement_protocol, KeyAgreementProtocol::Curve25519);
        assert_eq!(sas.message_authentication_code, MessageAuthenticationCode::HkdfHmacSha256V2);
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);
    }

    #[test]
    fn in_room_serialization_roundtrip() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let content = KeyVerificationAcceptEventContent {
            relates_to: Reference { event_id: event_id.to_owned() },
            method: AcceptMethod::SasV1(SasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256V2,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: Base64::new(b"hello".to_vec()),
            }),
        };

        let json_content = Raw::new(&content).unwrap();
        let deser_content = json_content.deserialize().unwrap();

        assert_matches!(deser_content.method, AcceptMethod::SasV1(_));
        assert_eq!(deser_content.relates_to.event_id, event_id);
    }

    #[test]
    fn custom_to_device_serialization_roundtrip() {
        let json = json!({
            "transaction_id": "456",
            "method": "m.sas.custom",
            "test": "field",
        });

        let content =
            from_json_value::<ToDeviceKeyVerificationAcceptEventContent>(json.clone()).unwrap();

        assert_eq!(content.transaction_id, "456");
        assert_eq!(content.method.method(), "m.sas.custom");
        let data = &*content.method.data();
        assert_eq!(data.len(), 1);
        assert_let!(Some(JsonValue::String(value)) = data.get("test"));
        assert_eq!(value, "field");

        assert_to_canonical_json_eq!(content, json);
    }
}
