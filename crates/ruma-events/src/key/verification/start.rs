//! Types for the [`m.key.verification.start`] event.
//!
//! [`m.key.verification.start`]: https://spec.matrix.org/latest/client-server-api/#mkeyverificationstart

use std::{collections::BTreeMap, fmt};

use ruma_common::{serde::Base64, OwnedDeviceId, OwnedTransactionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
use crate::relation::Reference;

/// The content of a to-device `m.key.verification.start` event.
///
/// Begins an SAS key verification process.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.start", kind = ToDevice)]
pub struct ToDeviceKeyVerificationStartEventContent {
    /// The device ID which is initiating the process.
    pub from_device: OwnedDeviceId,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the `m.key.verification.request` if this process is originating
    /// from a request.
    pub transaction_id: OwnedTransactionId,

    /// Method specific content.
    #[serde(flatten)]
    pub method: StartMethod,
}

impl ToDeviceKeyVerificationStartEventContent {
    /// Creates a new `ToDeviceKeyVerificationStartEventContent` with the given device ID,
    /// transaction ID and method specific content.
    pub fn new(
        from_device: OwnedDeviceId,
        transaction_id: OwnedTransactionId,
        method: StartMethod,
    ) -> Self {
        Self { from_device, transaction_id, method }
    }
}

/// The content of an in-room `m.key.verification.start` event.
///
/// Begins an SAS key verification process.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.start", kind = MessageLike)]
pub struct KeyVerificationStartEventContent {
    /// The device ID which is initiating the process.
    pub from_device: OwnedDeviceId,

    /// Method specific content.
    #[serde(flatten)]
    pub method: StartMethod,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl KeyVerificationStartEventContent {
    /// Creates a new `KeyVerificationStartEventContent` with the given device ID, method and
    /// reference.
    pub fn new(from_device: OwnedDeviceId, method: StartMethod, relates_to: Reference) -> Self {
        Self { from_device, method, relates_to }
    }
}

/// An enum representing the different method specific `m.key.verification.start` content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum StartMethod {
    /// The `m.sas.v1` verification method.
    SasV1(SasV1Content),

    /// The `m.reciprocate.v1` verification method.
    ///
    /// The spec entry for this method can be found [here].
    ///
    /// [here]: https://spec.matrix.org/latest/client-server-api/#mkeyverificationstartmreciprocatev1
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

/// The payload of an `m.key.verification.start` event using the `m.sas.v1` method.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename = "m.reciprocate.v1", tag = "method")]
pub struct ReciprocateV1Content {
    /// The shared secret from the QR code, encoded using unpadded base64.
    pub secret: Base64,
}

impl ReciprocateV1Content {
    /// Create a new `ReciprocateV1Content` with the given shared secret.
    ///
    /// The shared secret needs to come from the scanned QR code, encoded using unpadded base64.
    pub fn new(secret: Base64) -> Self {
        Self { secret }
    }
}

impl fmt::Debug for ReciprocateV1Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReciprocateV1Content").finish_non_exhaustive()
    }
}

/// The payload of an `m.key.verification.start` event using the `m.sas.v1` method.
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
    /// Must include at least `hkdf-hmac-sha256.v2`. Should also include `hkdf-hmac-sha256` for
    /// compatibility with older clients, though this MAC is deprecated and will be removed in a
    /// future version of the spec.
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
    /// Must include at least `hkdf-hmac-sha256.v2`. Should also include `hkdf-hmac-sha256` for
    /// compatibility with older clients, though this MAC is deprecated and will be removed in a
    /// future version of the spec.
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

    use assert_matches2::assert_matches;
    use ruma_common::{event_id, serde::Base64};
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{
        HashAlgorithm, KeyAgreementProtocol, KeyVerificationStartEventContent,
        MessageAuthenticationCode, ReciprocateV1Content, SasV1ContentInit,
        ShortAuthenticationString, StartMethod, ToDeviceKeyVerificationStartEventContent,
        _CustomContent,
    };
    use crate::{relation::Reference, ToDeviceEvent};

    #[test]
    fn serialization() {
        let key_verification_start_content = ToDeviceKeyVerificationStartEventContent {
            from_device: "123".into(),
            transaction_id: "456".into(),
            method: StartMethod::SasV1(
                SasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256V2],
                    short_authentication_string: vec![ShortAuthenticationString::Decimal],
                }
                .into(),
            ),
        };

        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.v1",
            "key_agreement_protocols": ["curve25519"],
            "hashes": ["sha256"],
            "message_authentication_codes": ["hkdf-hmac-sha256.v2"],
            "short_authentication_string": ["decimal"]
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);

        let json_data = json!({
            "from_device": "123",
            "transaction_id": "456",
            "method": "m.sas.custom",
            "test": "field",
        });

        let key_verification_start_content = ToDeviceKeyVerificationStartEventContent {
            from_device: "123".into(),
            transaction_id: "456".into(),
            method: StartMethod::_Custom(_CustomContent {
                method: "m.sas.custom".to_owned(),
                data: vec![("test".to_owned(), JsonValue::from("field"))]
                    .into_iter()
                    .collect::<BTreeMap<String, JsonValue>>(),
            }),
        };

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);

        {
            let secret = Base64::new(b"This is a secret to everybody".to_vec());

            let key_verification_start_content = ToDeviceKeyVerificationStartEventContent {
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
    fn in_room_serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let key_verification_start_content = KeyVerificationStartEventContent {
            from_device: "123".into(),
            relates_to: Reference { event_id: event_id.to_owned() },
            method: StartMethod::SasV1(
                SasV1ContentInit {
                    hashes: vec![HashAlgorithm::Sha256],
                    key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                    message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256V2],
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
            "message_authentication_codes": ["hkdf-hmac-sha256.v2"],
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);

        let secret = Base64::new(b"This is a secret to everybody".to_vec());

        let key_verification_start_content = KeyVerificationStartEventContent {
            from_device: "123".into(),
            relates_to: Reference { event_id: event_id.to_owned() },
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
            "message_authentication_codes": ["hkdf-hmac-sha256.v2"],
            "short_authentication_string": ["decimal"]
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        let content = from_json_value::<ToDeviceKeyVerificationStartEventContent>(json).unwrap();
        assert_eq!(content.from_device, "123");
        assert_eq!(content.transaction_id, "456");

        assert_matches!(content.method, StartMethod::SasV1(sas));
        assert_eq!(sas.hashes, vec![HashAlgorithm::Sha256]);
        assert_eq!(sas.key_agreement_protocols, vec![KeyAgreementProtocol::Curve25519]);
        assert_eq!(
            sas.message_authentication_codes,
            vec![MessageAuthenticationCode::HkdfHmacSha256V2]
        );
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);

        let json = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.v1",
                "key_agreement_protocols": ["curve25519"],
                "hashes": ["sha256"],
                "message_authentication_codes": ["hkdf-hmac-sha256.v2"],
                "short_authentication_string": ["decimal"]
            },
            "type": "m.key.verification.start",
            "sender": "@example:localhost",
        });

        let ev = from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationStartEventContent>>(json)
            .unwrap();
        assert_eq!(ev.sender, "@example:localhost");
        assert_eq!(ev.content.from_device, "123");
        assert_eq!(ev.content.transaction_id, "456");

        assert_matches!(ev.content.method, StartMethod::SasV1(sas));
        assert_eq!(sas.hashes, vec![HashAlgorithm::Sha256]);
        assert_eq!(sas.key_agreement_protocols, vec![KeyAgreementProtocol::Curve25519]);
        assert_eq!(
            sas.message_authentication_codes,
            vec![MessageAuthenticationCode::HkdfHmacSha256V2]
        );
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);

        let json = json!({
            "content": {
                "from_device": "123",
                "transaction_id": "456",
                "method": "m.sas.custom",
                "test": "field",
            },
            "type": "m.key.verification.start",
            "sender": "@example:localhost",
        });

        let ev = from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationStartEventContent>>(json)
            .unwrap();
        assert_eq!(ev.sender, "@example:localhost");
        assert_eq!(ev.content.from_device, "123");
        assert_eq!(ev.content.transaction_id, "456");

        assert_matches!(ev.content.method, StartMethod::_Custom(custom));
        assert_eq!(custom.method, "m.sas.custom");
        assert_eq!(custom.data.get("test"), Some(&JsonValue::from("field")));

        let json = json!({
            "content": {
                "from_device": "123",
                "method": "m.reciprocate.v1",
                "secret": "c2VjcmV0Cg",
                "transaction_id": "456",
            },
            "type": "m.key.verification.start",
            "sender": "@example:localhost",
        });

        let ev = from_json_value::<ToDeviceEvent<ToDeviceKeyVerificationStartEventContent>>(json)
            .unwrap();
        assert_eq!(ev.sender, "@example:localhost");
        assert_eq!(ev.content.from_device, "123");
        assert_eq!(ev.content.transaction_id, "456");

        assert_matches!(ev.content.method, StartMethod::ReciprocateV1(reciprocate));
        assert_eq!(reciprocate.secret.encode(), "c2VjcmV0Cg");
    }

    #[test]
    fn in_room_deserialization() {
        let json = json!({
            "from_device": "123",
            "method": "m.sas.v1",
            "hashes": ["sha256"],
            "key_agreement_protocols": ["curve25519"],
            "message_authentication_codes": ["hkdf-hmac-sha256.v2"],
            "short_authentication_string": ["decimal"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$1598361704261elfgc:localhost",
            }
        });

        // Deserialize the content struct separately to verify `TryFromRaw` is implemented for it.
        let content = from_json_value::<KeyVerificationStartEventContent>(json).unwrap();
        assert_eq!(content.from_device, "123");
        assert_eq!(content.relates_to.event_id, "$1598361704261elfgc:localhost");

        assert_matches!(content.method, StartMethod::SasV1(sas));
        assert_eq!(sas.hashes, vec![HashAlgorithm::Sha256]);
        assert_eq!(sas.key_agreement_protocols, vec![KeyAgreementProtocol::Curve25519]);
        assert_eq!(
            sas.message_authentication_codes,
            vec![MessageAuthenticationCode::HkdfHmacSha256V2]
        );
        assert_eq!(sas.short_authentication_string, vec![ShortAuthenticationString::Decimal]);

        let json = json!({
            "from_device": "123",
            "method": "m.reciprocate.v1",
            "secret": "c2VjcmV0Cg",
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$1598361704261elfgc:localhost",
            }
        });

        let content = from_json_value::<KeyVerificationStartEventContent>(json).unwrap();
        assert_eq!(content.from_device, "123");
        assert_eq!(content.relates_to.event_id, "$1598361704261elfgc:localhost");

        assert_matches!(content.method, StartMethod::ReciprocateV1(reciprocate));
        assert_eq!(reciprocate.secret.encode(), "c2VjcmV0Cg");
    }
}
