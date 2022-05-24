//! Types for the [`m.key.verification.accept`] event.
//!
//! [`m.key.verification.accept`]: https://spec.matrix.org/v1.2/client-server-api/#mkeyverificationaccept

use std::collections::BTreeMap;

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, Relation,
    ShortAuthenticationString,
};
use crate::{serde::Base64, OwnedTransactionId};

/// The content of a to-device `m.key.verification.accept` event.
///
/// Accepts a previously sent `m.key.verification.start` message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct KeyVerificationAcceptEventContent {
    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

impl KeyVerificationAcceptEventContent {
    /// Creates a new `ToDeviceKeyVerificationAcceptEventContent` with the given method-specific
    /// content and relation.
    pub fn new(method: AcceptMethod, relates_to: Relation) -> Self {
        Self { method, relates_to }
    }
}

/// An enum representing the different method specific `m.key.verification.accept` content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum AcceptMethod {
    /// The `m.sas.v1` verification method.
    SasV1(SasV1Content),

    /// Any unknown accept method.
    #[doc(hidden)]
    _Custom(_CustomContent),
}

/// Method specific content of a unknown key verification method.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct _CustomContent {
    /// The name of the method.
    pub method: String,

    /// The additional fields that the method contains.
    #[serde(flatten)]
    pub data: BTreeMap<String, JsonValue>,
}

/// The payload of an `m.key.verification.accept` event using the `m.sas.v1` method.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    use std::collections::BTreeMap;

    use crate::{event_id, serde::Base64, user_id};
    use assert_matches::assert_matches;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{
        AcceptMethod, HashAlgorithm, KeyAgreementProtocol, KeyVerificationAcceptEventContent,
        MessageAuthenticationCode, SasV1Content, ShortAuthenticationString,
        ToDeviceKeyVerificationAcceptEventContent, _CustomContent,
    };
    use crate::events::{key::verification::Relation, ToDeviceEvent};

    #[test]
    fn serialization() {
        let key_verification_accept_content = ToDeviceKeyVerificationAcceptEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::SasV1(SasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: Base64::new(b"hello".to_vec()),
            }),
        };

        let sender = user_id!("@example:localhost").to_owned();

        let json_data = json!({
            "content": {
                "transaction_id": "456",
                "method": "m.sas.v1",
                "commitment": "aGVsbG8",
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

        let sender = user_id!("@example:localhost").to_owned();

        let json_data = json!({
            "content": {
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "sender": sender,
            "type": "m.key.verification.accept"
        });

        let key_verification_accept_content = ToDeviceKeyVerificationAcceptEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::_Custom(_CustomContent {
                method: "m.sas.custom".to_owned(),
                data: vec![("test".to_owned(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        let key_verification_accept =
            ToDeviceEvent { sender, content: key_verification_accept_content };

        assert_eq!(to_json_value(&key_verification_accept).unwrap(), json_data);
    }

    #[test]
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_accept_content = KeyVerificationAcceptEventContent {
            relates_to: Relation { event_id: event_id.to_owned() },
            method: AcceptMethod::SasV1(SasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: Base64::new(b"hello".to_vec()),
            }),
        };

        let json_data = json!({
            "method": "m.sas.v1",
            "commitment": "aGVsbG8",
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
            "commitment": "aGVsbG8",
            "method": "m.sas.v1",
            "hash": "sha256",
            "key_agreement_protocol": "curve25519",
            "message_authentication_code": "hkdf-hmac-sha256",
            "short_authentication_string": ["decimal"]
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        assert_matches!(
            from_json_value::<ToDeviceKeyVerificationAcceptEventContent>(json).unwrap(),
            ToDeviceKeyVerificationAcceptEventContent {
                transaction_id,
                method: AcceptMethod::SasV1(SasV1Content {
                    commitment,
                    hash,
                    key_agreement_protocol,
                    message_authentication_code,
                    short_authentication_string,
                })
            } if commitment.encode() == "aGVsbG8"
                && transaction_id == "456"
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

        let sender = user_id!("@example:localhost");

        let json = json!({
            "content": {
                "commitment": "aGVsbG8",
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
            from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationAcceptEventContent>>(json).unwrap(),
            ToDeviceEvent {
                sender,
                content: ToDeviceKeyVerificationAcceptEventContent {
                    transaction_id,
                    method: AcceptMethod::SasV1(SasV1Content {
                        commitment,
                        hash,
                        key_agreement_protocol,
                        message_authentication_code,
                        short_authentication_string,
                    })
                }
            } if commitment.encode() == "aGVsbG8"
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
            from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationAcceptEventContent>>(json).unwrap(),
            ToDeviceEvent {
                sender,
                content: ToDeviceKeyVerificationAcceptEventContent {
                    transaction_id,
                    method: AcceptMethod::_Custom(_CustomContent {
                        method,
                        data,
                    })
                }
            } if transaction_id == "456"
                && sender == user_id!("@example:localhost")
                && method == "m.sas.custom"
                && data.get("test").unwrap() == &JsonValue::from("field")
        );
    }

    #[test]
    fn in_room_deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "commitment": "aGVsbG8",
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
            from_json_value::<KeyVerificationAcceptEventContent>(json).unwrap(),
            KeyVerificationAcceptEventContent {
                relates_to: Relation {
                    event_id
                },
                method: AcceptMethod::SasV1(SasV1Content {
                    commitment,
                    hash,
                    key_agreement_protocol,
                    message_authentication_code,
                    short_authentication_string,
                })
            } if commitment.encode() == "aGVsbG8"
                && *event_id == *id
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );
    }
}
