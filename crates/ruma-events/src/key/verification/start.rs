//! Types for the *m.key.verification.start* event.

use std::collections::BTreeMap;

use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Begins an SAS key verification process.
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub type StartEvent = MessageEvent<StartEventContent>;

/// The payload of a to-device *m.key.verification.start* event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.start", kind = ToDevice)]
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

impl StartToDeviceEventContent {
    /// Creates a new `StartToDeviceEventContent` with the given device ID, transaction ID and
    /// method specific content.
    pub fn new(from_device: DeviceIdBox, transaction_id: String, method: StartMethod) -> Self {
        Self { from_device, transaction_id, method }
    }
}

/// The payload of an in-room *m.key.verification.start* event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.start", kind = Message)]
pub struct StartEventContent {
    /// The device ID which is initiating the process.
    pub from_device: DeviceIdBox,

    /// Method specific content.
    #[serde(flatten)]
    pub method: StartMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

#[cfg(feature = "unstable-pre-spec")]
impl StartEventContent {
    /// Creates a new `StartEventContent` with the given device ID, method and relation.
    pub fn new(from_device: DeviceIdBox, method: StartMethod, relates_to: Relation) -> Self {
        Self { from_device, method, relates_to }
    }
}

/// An enum representing the different method specific *m.key.verification.start* content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum StartMethod {
    /// The *m.sas.v1* verification method.
    SasV1(SasV1Content),

    /// The *m.reciprocate.v1* verification method.
    ///
    /// The spec entry for this method can be found [here][1].
    ///
    /// [1]: https://spec.matrix.org/unstable/client-server-api/#mkeyverificationstartmreciprocatev1
    #[cfg(feature = "unstable-pre-spec")]
    ReciprocateV1(ReciprocateV1Content),

    /// Any unknown start method.
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

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename = "m.reciprocate.v1", tag = "method")]
pub struct ReciprocateV1Content {
    /// The shared secret from the QR code, encoded using unpadded base64.
    pub secret: String,
}

#[cfg(feature = "unstable-pre-spec")]
impl ReciprocateV1Content {
    /// Create a new `ReciprocateV1Content` with the given shared secret.
    ///
    /// The shared secret needs to come from the scanned QR code, encoded using
    /// unpadded base64.
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
///
/// To create an instance of this type, first create a `SasV1ContentInit` and convert it via
/// `SasV1Content::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename = "m.sas.v1", tag = "method")]
pub struct SasV1Content {
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

/// Mandatory initial set of fields for creating an `SasV1Content`.
///
/// This struct will not be updated even if additional fields are added to `SasV1Content` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SasV1ContentInit {
    /// The key agreement protocols the sending device understands.
    ///
    /// Should include at least `curve25519`.
    pub key_agreement_protocols: Vec<KeyAgreementProtocol>,

    /// The hash methods the sending device understands.
    ///
    /// Should include at least `sha256`.
    pub hashes: Vec<HashAlgorithm>,

    /// The message authentication codes that the sending device understands.
    ///
    /// Should include at least `hkdf-hmac-sha256`.
    pub message_authentication_codes: Vec<MessageAuthenticationCode>,

    /// The SAS methods the sending device (and the sending device's user) understands.
    ///
    /// Should include at least `decimal`.
    pub short_authentication_string: Vec<ShortAuthenticationString>,
}

impl From<SasV1ContentInit> for SasV1Content {
    /// Creates a new `SasV1Content` from the given init struct.
    fn from(init: SasV1ContentInit) -> Self {
        Self {
            key_agreement_protocols: init.key_agreement_protocols,
            hashes: init.hashes,
            message_authentication_codes: init.message_authentication_codes,
            short_authentication_string: init.short_authentication_string,
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

    use super::{
        HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, SasV1Content,
        SasV1ContentInit, ShortAuthenticationString, StartMethod, StartToDeviceEventContent,
        _CustomContent,
    };
    #[cfg(feature = "unstable-pre-spec")]
    use super::{ReciprocateV1Content, StartEventContent};
    #[cfg(feature = "unstable-pre-spec")]
    use crate::key::verification::Relation;
    use crate::ToDeviceEvent;

    #[test]
    fn serialization() {
        let key_verification_start_content = StartToDeviceEventContent {
            from_device: "123".into(),
            transaction_id: "456".into(),
            method: StartMethod::SasV1(
                SasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
                    short_authentication_string: vec![ShortAuthenticationString::Decimal],
                }
                .into(),
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
            method: StartMethod::_Custom(_CustomContent {
                method: "m.sas.custom".to_owned(),
                data: vec![("test".to_owned(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        let key_verification_start =
            ToDeviceEvent { sender, content: key_verification_start_content };

        assert_eq!(to_json_value(&key_verification_start).unwrap(), json_data);

        #[cfg(feature = "unstable-pre-spec")]
        {
            let secret = "This is a secret to everybody".to_owned();

            let key_verification_start_content = StartToDeviceEventContent {
                from_device: "123".into(),
                transaction_id: "456".into(),
                method: StartMethod::ReciprocateV1(ReciprocateV1Content::new(secret.clone())),
            };

            let json_data = json!({
                "from_device": "123",
                "method": "m.reciprocate.v1",
                "secret": secret,
                "transaction_id": "456"
            });

            assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);
        }
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_start_content = StartEventContent {
            from_device: "123".into(),
            relates_to: Relation { event_id: event_id.clone() },
            method: StartMethod::SasV1(
                SasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
                    short_authentication_string: vec![ShortAuthenticationString::Decimal],
                }
                .into(),
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

        let secret = "This is a secret to everybody".to_owned();

        let key_verification_start_content = StartEventContent {
            from_device: "123".into(),
            relates_to: Relation { event_id: event_id.clone() },
            method: StartMethod::ReciprocateV1(ReciprocateV1Content::new(secret.clone())),
        };

        let json_data = json!({
            "from_device": "123",
            "method": "m.reciprocate.v1",
            "secret": secret,
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
                method: StartMethod::SasV1(SasV1Content {
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
                    method: StartMethod::SasV1(SasV1Content {
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
                    method: StartMethod::_Custom(_CustomContent { method, data })
                }
            } if from_device == "123"
                && sender == user_id!("@example:localhost")
                && transaction_id == "456"
                && method == "m.sas.custom"
                && data.get("test").unwrap() == &JsonValue::from("field")
        );

        #[cfg(feature = "unstable-pre-spec")]
        {
            let json = json!({
                "content": {
                    "from_device": "123",
                    "method": "m.reciprocate.v1",
                    "secret": "It's a secret to everybody",
                    "transaction_id": "456",
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
                        method: StartMethod::ReciprocateV1(ReciprocateV1Content { secret }),
                    }
                } if from_device == "123"
                    && sender == user_id!("@example:localhost")
                    && transaction_id == "456"
                    && secret == "It's a secret to everybody"
            );
        }
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
                relates_to: Relation { event_id },
                method: StartMethod::SasV1(SasV1Content {
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

        let json = json!({
            "from_device": "123",
            "method": "m.reciprocate.v1",
            "secret": "It's a secret to everybody",
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        assert_matches!(
            from_json_value::<Raw<StartEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StartEventContent {
                from_device,
                relates_to: Relation { event_id },
                method: StartMethod::ReciprocateV1(ReciprocateV1Content { secret }),
            } if from_device == "123"
                && event_id == id
                && secret == "It's a secret to everybody"
        );
    }
}
