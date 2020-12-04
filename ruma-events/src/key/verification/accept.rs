//! Types for the *m.key.verification.accept* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Accepts a previously sent *m.key.verification.start* message.
#[cfg(feature = "unstable-pre-spec")]
pub type AcceptEvent = MessageEvent<AcceptEventContent>;

/// The payload for a to-device `AcceptEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.accept")]
pub struct AcceptToDeviceEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start*
    /// message.
    pub transaction_id: String,

    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,
}

/// The payload for a in-room `AcceptEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.accept")]
#[cfg(feature = "unstable-pre-spec")]
pub struct AcceptEventContent {
    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

/// An enum representing the different method specific
/// *m.key.verification.accept* content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum AcceptMethod {
    /// The *m.sas.v1* verification method.
    MSasV1(MSasV1Content),

    /// Any unknown accept method.
    Custom(CustomContent),
}

/// Method specific content of a unknown key verification method.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomContent {
    /// The name of the method.
    pub method: String,

    /// The additional fields that the method contains.
    #[serde(flatten)]
    pub fields: BTreeMap<String, JsonValue>,
}

/// The payload of an *m.key.verification.accept* event using the *m.sas.v1* method.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename = "m.sas.v1", tag = "method")]
pub struct MSasV1Content {
    /// The key agreement protocol the device is choosing to use, out of the
    /// options in the *m.key.verification.start* message.
    pub key_agreement_protocol: KeyAgreementProtocol,

    /// The hash method the device is choosing to use, out of the options in the
    /// *m.key.verification.start* message.
    pub hash: HashAlgorithm,

    /// The message authentication code the device is choosing to use, out of
    /// the options in the *m.key.verification.start* message.
    pub message_authentication_code: MessageAuthenticationCode,

    /// The SAS methods both devices involved in the verification process
    /// understand.
    ///
    /// Must be a subset of the options in the *m.key.verification.start*
    /// message.
    pub short_authentication_string: Vec<ShortAuthenticationString>,

    /// The hash (encoded as unpadded base64) of the concatenation of the
    /// device's ephemeral public key (encoded as unpadded base64) and the
    /// canonical JSON representation of the *m.key.verification.start* message.
    pub commitment: String,
}

/// Mandatory initial set of fields for creating an accept `MSasV1Content`.
#[derive(Clone, Debug, Deserialize)]
pub struct MSasV1ContentInit {
    /// The key agreement protocol the device is choosing to use, out of the
    /// options in the *m.key.verification.start* message.
    pub key_agreement_protocol: KeyAgreementProtocol,

    /// The hash method the device is choosing to use, out of the options in the
    /// *m.key.verification.start* message.
    pub hash: HashAlgorithm,

    /// The message authentication codes that the accepting device understands.
    pub message_authentication_code: MessageAuthenticationCode,

    /// The SAS methods both devices involved in the verification process
    /// understand.
    ///
    /// Must be a subset of the options in the *m.key.verification.start*
    /// message.
    pub short_authentication_string: Vec<ShortAuthenticationString>,

    /// The hash (encoded as unpadded base64) of the concatenation of the
    /// device's ephemeral public key (encoded as unpadded base64) and the
    /// canonical JSON representation of the *m.key.verification.start* message.
    pub commitment: String,
}

impl From<MSasV1ContentInit> for MSasV1Content {
    /// Creates a new `MSasV1Content` from the given init struct.
    fn from(init: MSasV1ContentInit) -> Self {
        MSasV1Content {
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
    use std::collections::BTreeMap;

    use matches::assert_matches;
    #[cfg(feature = "unstable-pre-spec")]
    use ruma_identifiers::event_id;
    use ruma_identifiers::user_id;
    use ruma_serde::Raw;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    #[cfg(feature = "unstable-pre-spec")]
    use super::{AcceptEventContent, Relation};
    use super::{
        AcceptMethod, AcceptToDeviceEventContent, CustomContent, HashAlgorithm,
        KeyAgreementProtocol, MSasV1Content, MessageAuthenticationCode, ShortAuthenticationString,
    };
    use crate::ToDeviceEvent;

    #[test]
    fn serialization() {
        let key_verification_accept_content = AcceptToDeviceEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::MSasV1(MSasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: "test_commitment".into(),
            }),
        };

        let sender = user_id!("@example:localhost");

        let json_data = json!({
            "content": {
                "transaction_id": "456",
                "method": "m.sas.v1",
                "commitment": "test_commitment",
                "key_agreement_protocol": "curve25519",
                "hash": "sha256",
                "message_authentication_code": "hkdf-hmac-sha256",
                "short_authentication_string": ["decimal"]
            },
            "sender": sender,
            "type": "m.key.verification.accept"
        });

        let key_verification_accept =
            ToDeviceEvent { sender, content: key_verification_accept_content };

        assert_eq!(to_json_value(&key_verification_accept).unwrap(), json_data);

        let sender = user_id!("@example:localhost");

        let json_data = json!({
            "content": {
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "sender": sender,
            "type": "m.key.verification.accept"
        });

        let key_verification_accept_content = AcceptToDeviceEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::Custom(CustomContent {
                method: "m.sas.custom".to_owned(),
                fields: vec![("test".to_string(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        let key_verification_accept =
            ToDeviceEvent { sender, content: key_verification_accept_content };

        assert_eq!(to_json_value(&key_verification_accept).unwrap(), json_data);
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_accept_content = AcceptEventContent {
            relation: Relation { event_id: event_id.clone() },
            method: AcceptMethod::MSasV1(MSasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: "test_commitment".into(),
            }),
        };

        let json_data = json!({
            "method": "m.sas.v1",
            "commitment": "test_commitment",
            "key_agreement_protocol": "curve25519",
            "hash": "sha256",
            "message_authentication_code": "hkdf-hmac-sha256",
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        assert_eq!(to_json_value(&key_verification_accept_content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "transaction_id": "456",
            "commitment": "test_commitment",
            "method": "m.sas.v1",
            "hash": "sha256",
            "key_agreement_protocol": "curve25519",
            "message_authentication_code": "hkdf-hmac-sha256",
            "short_authentication_string": ["decimal"]
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        assert_matches!(
            from_json_value::<Raw<AcceptToDeviceEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AcceptToDeviceEventContent {
                transaction_id,
                method: AcceptMethod::MSasV1(MSasV1Content {
                    commitment,
                    hash,
                    key_agreement_protocol,
                    message_authentication_code,
                    short_authentication_string,
                })
            } if commitment == "test_commitment"
                && transaction_id == "456"
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

        let sender = user_id!("@example:localhost");

        let json = json!({
            "content": {
                "commitment": "test_commitment",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocol": "curve25519",
                "hash": "sha256",
                "message_authentication_code": "hkdf-hmac-sha256",
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.accept",
            "sender": sender,
        });

        assert_matches!(
            from_json_value::<Raw<ToDeviceEvent<AcceptToDeviceEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            ToDeviceEvent {
                sender,
                content: AcceptToDeviceEventContent {
                    transaction_id,
                    method: AcceptMethod::MSasV1(MSasV1Content {
                        commitment,
                        hash,
                        key_agreement_protocol,
                        message_authentication_code,
                        short_authentication_string,
                    })
                }
            } if commitment == "test_commitment"
                && sender == user_id!("@example:localhost")
                && transaction_id == "456"
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

        let sender = user_id!("@example:localhost");

        let json = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "type": "m.key.verification.accept",
            "sender": sender
        });

        assert_matches!(
            from_json_value::<Raw<ToDeviceEvent<AcceptToDeviceEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            ToDeviceEvent {
                sender,
                content: AcceptToDeviceEventContent {
                    transaction_id,
                    method: AcceptMethod::Custom(CustomContent {
                        method,
                        fields,
                    })
                }
            } if transaction_id == "456"
                && sender == user_id!("@example:localhost")
                && method == "m.sas.custom"
                && fields.get("test").unwrap() == &JsonValue::from("field")
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn in_room_deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "commitment": "test_commitment",
            "method": "m.sas.v1",
            "hash": "sha256",
            "key_agreement_protocol": "curve25519",
            "message_authentication_code": "hkdf-hmac-sha256",
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        assert_matches!(
            from_json_value::<Raw<AcceptEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AcceptEventContent {
                relation: Relation {
                    event_id
                },
                method: AcceptMethod::MSasV1(MSasV1Content {
                    commitment,
                    hash,
                    key_agreement_protocol,
                    message_authentication_code,
                    short_authentication_string,
                })
            } if commitment == "test_commitment"
                && event_id == id
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );
    }
}
