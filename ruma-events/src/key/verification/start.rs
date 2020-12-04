//! Types for the *m.key.verification.start* event.

use std::{collections::BTreeMap, convert::TryFrom};

use ruma_events_macros::BasicEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events_macros::MessageEventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
use crate::InvalidInput;
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Begins an SAS key verification process.
#[cfg(feature = "unstable-pre-spec")]
pub type StartEvent = MessageEvent<StartEventContent>;

/// The payload of a to-device *m.key.verification.start* event.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.start")]
pub struct StartToDeviceEventContent {
    /// The device ID which is initiating the process.
    pub from_device: DeviceIdBox,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the *m.key.verification.request* if this process is originating
    /// from a request.
    pub transaction_id: String,

    /// Method specific content.
    #[serde(flatten)]
    pub method: StartMethod,
}

/// The payload of an in-room *m.key.verification.start* event.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.start")]
#[cfg(feature = "unstable-pre-spec")]
pub struct StartEventContent {
    /// The device ID which is initiating the process.
    pub from_device: DeviceIdBox,

    /// Method specific content.
    #[serde(flatten)]
    pub method: StartMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

/// An enum representing the different method specific
/// *m.key.verification.start* content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum StartMethod {
    /// The *m.sas.v1* verification method.
    MSasV1(MSasV1Content),

    /// Any unknown start method.
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

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename = "m.sas.v1", tag = "method")]
pub struct MSasV1Content {
    /// The key agreement protocols the sending device understands.
    ///
    /// Must include at least `Curve25519` or `Curve25519HkdfSha256`.
    pub key_agreement_protocols: Vec<KeyAgreementProtocol>,

    /// The hash methods the sending device understands.
    ///
    /// Must include at least `sha256`.
    pub hashes: Vec<HashAlgorithm>,

    /// The message authentication codes that the sending device understands.
    ///
    /// Must include at least `hkdf-hmac-sha256`.
    pub message_authentication_codes: Vec<MessageAuthenticationCode>,

    /// The SAS methods the sending device (and the sending device's user) understands.
    ///
    /// Must include at least `decimal`. Optionally can include `emoji`.
    pub short_authentication_string: Vec<ShortAuthenticationString>,
}

/// Mandatory initial set of fields for creating an `MSasV1Content`.
#[derive(Clone, Debug, Deserialize)]
pub struct MSasV1ContentInit {
    /// The key agreement protocols the sending device understands.
    ///
    /// Must include at least `curve25519`.
    pub key_agreement_protocols: Vec<KeyAgreementProtocol>,

    /// The hash methods the sending device understands.
    ///
    /// Must include at least `sha256`.
    pub hashes: Vec<HashAlgorithm>,

    /// The message authentication codes that the sending device understands.
    ///
    /// Must include at least `hkdf-hmac-sha256`.
    pub message_authentication_codes: Vec<MessageAuthenticationCode>,

    /// The SAS methods the sending device (and the sending device's user) understands.
    ///
    /// Must include at least `decimal`. Optionally can include `emoji`.
    pub short_authentication_string: Vec<ShortAuthenticationString>,
}

impl MSasV1Content {
    /// Create a new `MSasV1Content` with the given values.
    ///
    /// # Errors
    ///
    /// `InvalidInput` will be returned in the following cases:
    ///
    /// * `key_agreement_protocols` does not include `KeyAgreementProtocol::Curve25519`.
    /// * `hashes` does not include `HashAlgorithm::Sha256`.
    /// * `message_authentication_codes` does not include
    /// `MessageAuthenticationCode::HkdfHmacSha256`.
    /// * `short_authentication_string` does not include `ShortAuthenticationString::Decimal`.
    pub fn new(options: MSasV1ContentInit) -> Result<Self, InvalidInput> {
        MSasV1Content::try_from(options)
    }
}

impl TryFrom<MSasV1ContentInit> for MSasV1Content {
    type Error = InvalidInput;

    /// Creates a new `MSasV1Content` from the given init struct.
    fn try_from(init: MSasV1ContentInit) -> Result<Self, Self::Error> {
        if !init.key_agreement_protocols.contains(&KeyAgreementProtocol::Curve25519)
            && !init.key_agreement_protocols.contains(&KeyAgreementProtocol::Curve25519HkdfSha256)
        {
            return Err(InvalidInput(
                "`key_agreement_protocols` must contain at \
                 least `KeyAgreementProtocol::Curve25519` or \
                 `KeyAgreementProtocol::Curve25519HkdfSha256`"
                    .into(),
            ));
        }

        if !init.hashes.contains(&HashAlgorithm::Sha256) {
            return Err(InvalidInput(
                "`hashes` must contain at least `HashAlgorithm::Sha256`".into(),
            ));
        }

        if !init.message_authentication_codes.contains(&MessageAuthenticationCode::HkdfHmacSha256) {
            return Err(InvalidInput(
                "`message_authentication_codes` must contain \
                 at least `MessageAuthenticationCode::HkdfHmacSha256`"
                    .into(),
            ));
        }

        if !init.short_authentication_string.contains(&ShortAuthenticationString::Decimal) {
            return Err(InvalidInput(
                "`short_authentication_string` must contain \
                 at least `ShortAuthenticationString::Decimal`"
                    .into(),
            ));
        }

        Ok(Self {
            key_agreement_protocols: init.key_agreement_protocols,
            hashes: init.hashes,
            message_authentication_codes: init.message_authentication_codes,
            short_authentication_string: init.short_authentication_string,
        })
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

    use super::{
        CustomContent, HashAlgorithm, KeyAgreementProtocol, MSasV1Content, MSasV1ContentInit,
        MessageAuthenticationCode, ShortAuthenticationString, StartMethod,
        StartToDeviceEventContent,
    };
    #[cfg(feature = "unstable-pre-spec")]
    use super::{Relation, StartEventContent};
    use crate::ToDeviceEvent;

    #[test]
    fn invalid_m_sas_v1_content_missing_required_key_agreement_protocols() {
        let error = MSasV1Content::new(MSasV1ContentInit {
            hashes: vec![HashAlgorithm::Sha256],
            key_agreement_protocols: vec![],
            message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
            short_authentication_string: vec![ShortAuthenticationString::Decimal],
        })
        .err()
        .unwrap();

        assert!(error.to_string().contains("key_agreement_protocols"));
    }

    #[test]
    fn invalid_m_sas_v1_content_missing_required_hashes() {
        let error = MSasV1Content::new(MSasV1ContentInit {
            hashes: vec![],
            key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
            message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
            short_authentication_string: vec![ShortAuthenticationString::Decimal],
        })
        .err()
        .unwrap();

        assert!(error.to_string().contains("hashes"));
    }

    #[test]
    fn invalid_m_sas_v1_content_missing_required_message_authentication_codes() {
        let error = MSasV1Content::new(MSasV1ContentInit {
            hashes: vec![HashAlgorithm::Sha256],
            key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
            message_authentication_codes: vec![],
            short_authentication_string: vec![ShortAuthenticationString::Decimal],
        })
        .err()
        .unwrap();

        assert!(error.to_string().contains("message_authentication_codes"));
    }

    #[test]
    fn invalid_m_sas_v1_content_missing_required_short_authentication_string() {
        let error = MSasV1Content::new(MSasV1ContentInit {
            hashes: vec![HashAlgorithm::Sha256],
            key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
            message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
            short_authentication_string: vec![],
        })
        .err()
        .unwrap();

        assert!(error.to_string().contains("short_authentication_string"));
    }

    #[test]
    fn serialization() {
        let key_verification_start_content = StartToDeviceEventContent {
            from_device: "123".into(),
            transaction_id: "456".into(),
            method: StartMethod::MSasV1(
                MSasV1Content::new(MSasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
                    short_authentication_string: vec![ShortAuthenticationString::Decimal],
                })
                .unwrap(),
            ),
        };

        let sender = user_id!("@example:localhost");

        let json_data = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocols": ["curve25519"],
                "hashes": ["sha256"],
                "message_authentication_codes": ["hkdf-hmac-sha256"],
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.start",
            "sender": sender
        });

        let key_verification_start =
            ToDeviceEvent { sender, content: key_verification_start_content };

        assert_eq!(to_json_value(&key_verification_start).unwrap(), json_data);

        let sender = user_id!("@example:localhost");

        let json_data = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "type": "m.key.verification.start",
            "sender": sender
        });

        let key_verification_start_content = StartToDeviceEventContent {
            from_device: "123".into(),
            transaction_id: "456".into(),
            method: StartMethod::Custom(CustomContent {
                method: "m.sas.custom".to_owned(),
                fields: vec![("test".to_string(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        let key_verification_start =
            ToDeviceEvent { sender, content: key_verification_start_content };

        assert_eq!(to_json_value(&key_verification_start).unwrap(), json_data);
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_start_content = StartEventContent {
            from_device: "123".into(),
            relation: Relation { event_id: event_id.clone() },
            method: StartMethod::MSasV1(
                MSasV1Content::new(MSasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
                    short_authentication_string: vec![ShortAuthenticationString::Decimal],
                })
                .unwrap(),
            ),
        };

        let json_data = json!({
            "from_device": "123",
            "method": "m.sas.v1",
            "key_agreement_protocols": ["curve25519"],
            "hashes": ["sha256"],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "hashes": ["sha256"],
            "key_agreement_protocols": ["curve25519"],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": ["decimal"]
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        assert_matches!(
            from_json_value::<Raw<StartToDeviceEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StartToDeviceEventContent {
                from_device,
                transaction_id,
                method: StartMethod::MSasV1(MSasV1Content {
                    hashes,
                    key_agreement_protocols,
                    message_authentication_codes,
                    short_authentication_string,
                })
            } if from_device == "123"
                && transaction_id == "456"
                && hashes == vec![HashAlgorithm::Sha256]
                && key_agreement_protocols == vec![KeyAgreementProtocol::Curve25519]
                && message_authentication_codes == vec![MessageAuthenticationCode::HkdfHmacSha256]
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

        let sender = user_id!("@example:localhost");

        let json = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocols": ["curve25519"],
                "hashes": ["sha256"],
                "message_authentication_codes": ["hkdf-hmac-sha256"],
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.start",
            "sender": sender
        });

        assert_matches!(
            from_json_value::<Raw<ToDeviceEvent<StartToDeviceEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            ToDeviceEvent {
                sender,
                content: StartToDeviceEventContent {
                    from_device,
                    transaction_id,
                    method: StartMethod::MSasV1(MSasV1Content {
                        hashes,
                        key_agreement_protocols,
                        message_authentication_codes,
                        short_authentication_string,
                    })
                }
            } if from_device == "123"
                && sender == user_id!("@example:localhost")
                && transaction_id == "456"
                && hashes == vec![HashAlgorithm::Sha256]
                && key_agreement_protocols == vec![KeyAgreementProtocol::Curve25519]
                && message_authentication_codes == vec![MessageAuthenticationCode::HkdfHmacSha256]
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
            "type": "m.key.verification.start",
            "sender": sender,
        });

        assert_matches!(
            from_json_value::<Raw<ToDeviceEvent<StartToDeviceEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            ToDeviceEvent {
                sender,
                content: StartToDeviceEventContent {
                    from_device,
                    transaction_id,
                    method: StartMethod::Custom(CustomContent {
                        method,
                        fields,
                    })
                }
            } if from_device == "123"
                && sender == user_id!("@example:localhost")
                && transaction_id == "456"
                && method == "m.sas.custom"
                && fields.get("test").unwrap() == &JsonValue::from("field")
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn in_room_deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "from_device": "123",
            "method": "m.sas.v1",
            "hashes": ["sha256"],
            "key_agreement_protocols": ["curve25519"],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        assert_matches!(
            from_json_value::<Raw<StartEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StartEventContent {
                from_device,
                relation: Relation {
                    event_id,
                },
                method: StartMethod::MSasV1(MSasV1Content {
                    hashes,
                    key_agreement_protocols,
                    message_authentication_codes,
                    short_authentication_string,
                })
            } if from_device == "123"
                && event_id == id
                && hashes == vec![HashAlgorithm::Sha256]
                && key_agreement_protocols == vec![KeyAgreementProtocol::Curve25519]
                && message_authentication_codes == vec![MessageAuthenticationCode::HkdfHmacSha256]
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );
    }

    #[test]
    fn deserialization_failure() {
        // Ensure that invalid JSON  creates a `serde_json::Error` and not `InvalidEvent`
        assert!(serde_json::from_str::<Raw<StartToDeviceEventContent>>("{").is_err());
    }

    // TODO this fails because the error is a Validation error not deserialization?
    /*
    #[test]
    fn deserialization_structure_mismatch() {
        // Missing several required fields.
        let error =
            from_json_value::<Raw<StartEventContent>>(json!({ "from_device": "123" }))
                .unwrap()
                .deserialize()
                .unwrap_err();

        assert!(error.message().contains("missing field"));
        assert!(error.is_deserialization());
    }
    */

    // TODO re implement validation done in TryFromRaw else where
    /*
    #[test]
    fn deserialization_validation_missing_required_key_agreement_protocols() {
        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "key_agreement_protocols": [],
            "hashes": ["sha256"],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": ["decimal"]
        });

        let error = from_json_value::<Raw<StartEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("key_agreement_protocols"));
        assert!(error.is_validation());
    }
    */

    // TODO re implement validation done in TryFromRaw else where
    /*
    #[test]
    fn deserialization_validation_missing_required_hashes() {
        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "key_agreement_protocols": ["curve25519"],
            "hashes": [],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": ["decimal"]
        });
        let error = from_json_value::<Raw<StartEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("hashes"));
        assert!(error.is_validation());
    }
    */

    // TODO re implement validation done in TryFromRaw else where
    /*
    #[test]
    fn deserialization_validation_missing_required_message_authentication_codes() {
        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "key_agreement_protocols": ["curve25519"],
            "hashes": ["sha256"],
            "message_authentication_codes": [],
            "short_authentication_string": ["decimal"]
        });
        let error = from_json_value::<Raw<StartEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("message_authentication_codes"));
        assert!(error.is_validation());
    }
    */

    /*
    #[test]
    fn deserialization_validation_missing_required_short_authentication_string() {
        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "key_agreement_protocols": ["curve25519"],
            "hashes": ["sha256"],
            "message_authentication_codes": ["hkdf-hmac-sha256"],
            "short_authentication_string": []
        });
        let error = from_json_value::<Raw<StartEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("short_authentication_string"));
        assert!(error.is_validation());
    }
    */

    // TODO re implement validation done in TryFromRaw else where
    /*
    #[test]
    fn deserialization_of_event_validates_content() {
        // This JSON is missing the required value of "curve25519" for "key_agreement_protocols".
        let json_data = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocols": [],
                "hashes": ["sha256"],
                "message_authentication_codes": ["hkdf-hmac-sha256"],
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.start"
        });
        let error = from_json_value::<Raw<StartEvent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("key_agreement_protocols"));
        assert!(error.is_validation());
    }
    **/
}
