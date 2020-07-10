//! Types for the *m.key.verification.start* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::DeviceId;
use serde::{Deserialize, Serialize};

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
};
use crate::{BasicEvent, InvalidInput};

/// Begins an SAS key verification process.
///
/// Typically sent as a to-device event.
pub type StartEvent = BasicEvent<StartEventContent>;

/// The payload of an *m.key.verification.start* event.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[non_exhaustive]
#[ruma_event(type = "m.key.verification.start")]
#[serde(tag = "method")]
pub enum StartEventContent {
    /// The *m.sas.v1* verification method.
    #[serde(rename = "m.sas.v1")]
    MSasV1(MSasV1Content),
}

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MSasV1Content {
    /// The device ID which is initiating the process.
    pub(crate) from_device: DeviceId,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the *m.key.verification.request* if this process is originating
    /// from a request.
    pub(crate) transaction_id: String,

    /// The key agreement protocols the sending device understands.
    ///
    /// Must include at least `curve25519`.
    pub(crate) key_agreement_protocols: Vec<KeyAgreementProtocol>,

    /// The hash methods the sending device understands.
    ///
    /// Must include at least `sha256`.
    pub(crate) hashes: Vec<HashAlgorithm>,

    /// The message authentication codes that the sending device understands.
    ///
    /// Must include at least `hkdf-hmac-sha256`.
    pub(crate) message_authentication_codes: Vec<MessageAuthenticationCode>,

    /// The SAS methods the sending device (and the sending device's user) understands.
    ///
    /// Must include at least `decimal`. Optionally can include `emoji`.
    pub(crate) short_authentication_string: Vec<ShortAuthenticationString>,
}

/// Options for creating an `MSasV1Content` with `MSasV1Content::new`.
#[derive(Clone, Debug, Deserialize)]
pub struct MSasV1ContentOptions {
    /// The device ID which is initiating the process.
    pub from_device: DeviceId,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the *m.key.verification.request* if this process is originating
    /// from a request.
    pub transaction_id: String,

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
    pub fn new(options: MSasV1ContentOptions) -> Result<Self, InvalidInput> {
        if !options.key_agreement_protocols.contains(&KeyAgreementProtocol::Curve25519) {
            return Err(InvalidInput("`key_agreement_protocols` must contain at least `KeyAgreementProtocol::Curve25519`".to_string()));
        }

        if !options.hashes.contains(&HashAlgorithm::Sha256) {
            return Err(InvalidInput(
                "`hashes` must contain at least `HashAlgorithm::Sha256`".to_string(),
            ));
        }

        if !options
            .message_authentication_codes
            .contains(&MessageAuthenticationCode::HkdfHmacSha256)
        {
            return Err(InvalidInput("`message_authentication_codes` must contain at least `MessageAuthenticationCode::HkdfHmacSha256`".to_string()));
        }

        if !options.short_authentication_string.contains(&ShortAuthenticationString::Decimal) {
            return Err(InvalidInput("`short_authentication_string` must contain at least `ShortAuthenticationString::Decimal`".to_string()));
        }

        Ok(Self {
            from_device: options.from_device,
            transaction_id: options.transaction_id,
            key_agreement_protocols: options.key_agreement_protocols,
            hashes: options.hashes,
            message_authentication_codes: options.message_authentication_codes,
            short_authentication_string: options.short_authentication_string,
        })
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        HashAlgorithm, KeyAgreementProtocol, MSasV1Content, MSasV1ContentOptions,
        MessageAuthenticationCode, ShortAuthenticationString, StartEvent, StartEventContent,
    };
    use crate::EventJson;

    #[test]
    fn invalid_m_sas_v1_content_missing_required_key_agreement_protocols() {
        let error = MSasV1Content::new(MSasV1ContentOptions {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
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
        let error = MSasV1Content::new(MSasV1ContentOptions {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
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
        let error = MSasV1Content::new(MSasV1ContentOptions {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
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
        let error = MSasV1Content::new(MSasV1ContentOptions {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
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
        let key_verification_start_content = StartEventContent::MSasV1(
            MSasV1Content::new(MSasV1ContentOptions {
                from_device: "123".to_string(),
                transaction_id: "456".to_string(),
                hashes: vec![HashAlgorithm::Sha256],
                key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
                message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
                short_authentication_string: vec![ShortAuthenticationString::Decimal],
            })
            .unwrap(),
        );

        let key_verification_start = StartEvent { content: key_verification_start_content };

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
            "type": "m.key.verification.start"
        });

        assert_eq!(to_json_value(&key_verification_start).unwrap(), json_data);
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
            from_json_value::<EventJson<StartEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StartEventContent::MSasV1(MSasV1Content {
                from_device,
                transaction_id,
                hashes,
                key_agreement_protocols,
                message_authentication_codes,
                short_authentication_string,
            }) if from_device == "123"
                && transaction_id == "456"
                && hashes == vec![HashAlgorithm::Sha256]
                && key_agreement_protocols == vec![KeyAgreementProtocol::Curve25519]
                && message_authentication_codes == vec![MessageAuthenticationCode::HkdfHmacSha256]
                && short_authentication_string == vec![ShortAuthenticationString::Decimal]
        );

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
            "type": "m.key.verification.start"
        });

        assert_matches!(
            from_json_value::<EventJson<StartEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StartEvent {
                content: StartEventContent::MSasV1(MSasV1Content {
                    from_device,
                    transaction_id,
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
        )
    }

    #[test]
    fn deserialization_failure() {
        // Ensure that invalid JSON  creates a `serde_json::Error` and not `InvalidEvent`
        assert!(serde_json::from_str::<EventJson<StartEventContent>>("{").is_err());
    }

    // TODO this fails because the error is a Validation error not deserialization?
    /*
    #[test]
    fn deserialization_structure_mismatch() {
        // Missing several required fields.
        let error =
            from_json_value::<EventJson<StartEventContent>>(json!({ "from_device": "123" }))
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

        let error = from_json_value::<EventJson<StartEventContent>>(json_data)
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
        let error = from_json_value::<EventJson<StartEventContent>>(json_data)
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
        let error = from_json_value::<EventJson<StartEventContent>>(json_data)
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
        let error = from_json_value::<EventJson<StartEventContent>>(json_data)
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
        let error = from_json_value::<EventJson<StartEvent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap_err();

        assert!(error.message().contains("key_agreement_protocols"));
        assert!(error.is_validation());
    }
    **/
}
