//! Types for the *m.key.verification.accept* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
use crate::BasicEvent;

/// Accepts a previously sent *m.key.verification.start* message.
///
/// Typically sent as a to-device event.
pub type AcceptEvent = BasicEvent<AcceptEventContent>;

/// The payload for `AcceptEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.accept")]
pub struct AcceptEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start*
    /// message.
    pub transaction_id: String,

    /// The method specific content.
    #[serde(flatten)]
    pub method: AcceptMethod,
}

/// An enum representing the different method specific
/// *m.key.verification.accept* content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use matches::assert_matches;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{
        AcceptEvent, AcceptEventContent, AcceptMethod, CustomContent, HashAlgorithm,
        KeyAgreementProtocol, MSasV1Content, MessageAuthenticationCode, ShortAuthenticationString,
    };
    use ruma_common::Raw;

    #[test]
    fn serialization() {
        let key_verification_accept_content = AcceptEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::MSasV1(MSasV1Content {
                hash: HashAlgorithm::Sha256,
                key_agreement_protocol: KeyAgreementProtocol::Curve25519,
                message_authentication_code: MessageAuthenticationCode::HkdfHmacSha256,
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
                commitment: "test_commitment".into(),
            }),
        };

        let key_verification_accept = AcceptEvent { content: key_verification_accept_content };

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
            "type": "m.key.verification.accept"
        });

        assert_eq!(to_json_value(&key_verification_accept).unwrap(), json_data);

        let json_data = json!({
            "content": {
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "type": "m.key.verification.accept"
        });

        let key_verification_accept_content = AcceptEventContent {
            transaction_id: "456".into(),
            method: AcceptMethod::Custom(CustomContent {
                method: "m.sas.custom".to_owned(),
                fields: vec![("test".to_string(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        let key_verification_accept = AcceptEvent { content: key_verification_accept_content };

        assert_eq!(to_json_value(&key_verification_accept).unwrap(), json_data);
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
            from_json_value::<Raw<AcceptEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AcceptEventContent {
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
            "type": "m.key.verification.accept"
        });

        assert_matches!(
            from_json_value::<Raw<AcceptEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AcceptEvent {
                content: AcceptEventContent {
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
                && transaction_id == "456"
                && hash == HashAlgorithm::Sha256
                && key_agreement_protocol == KeyAgreementProtocol::Curve25519
                && message_authentication_code == MessageAuthenticationCode::HkdfHmacSha256
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

        let json = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "type": "m.key.verification.accept"
        });

        assert_matches!(
            from_json_value::<Raw<AcceptEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AcceptEvent {
                content: AcceptEventContent {
                    transaction_id,
                    method: AcceptMethod::Custom(CustomContent {
                        method,
                        fields,
                    })
                }
            } if transaction_id == "456"
                && method == "m.sas.custom"
                && fields.get("test").unwrap() == &JsonValue::from("field")
        );
    }
}
