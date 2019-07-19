//! Types for the *m.key.verification.start* event.

use std::{convert::TryFrom, str::FromStr};

use ruma_identifiers::DeviceId;
use serde::{de::Error, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
    VerificationMethod,
};
use crate::{Event, EventType, InnerInvalidEvent, InvalidEvent, InvalidInput};

/// Begins an SAS key verification process.
///
/// Typically sent as a to-device event.
#[derive(Clone, Debug, PartialEq)]
pub struct StartEvent {
    /// The event's content.
    pub content: StartEventContent,
}

/// The payload of an *m.key.verification.start* event.
#[derive(Clone, Debug, PartialEq)]
pub enum StartEventContent {
    /// The *m.sas.v1* verification method.
    MSasV1(MSasV1Content),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl FromStr for StartEvent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::StartEvent>(json) {
            Ok(raw) => raw,
            Err(error) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: value,
                        message: error.to_string(),
                    }));
                }
                Err(error) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Deserialization { error }));
                }
            },
        };

        let content = match raw.content {
            raw::StartEventContent::MSasV1(content) => StartEventContent::MSasV1(content),
            raw::StartEventContent::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.");
            }
        };

        Ok(Self { content })
    }
}

impl<'a> TryFrom<&'a str> for StartEvent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for StartEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("StartEvent", 2)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("type", &self.event_type())?;

        state.end()
    }
}

impl_event!(
    StartEvent,
    StartEventContent,
    EventType::KeyVerificationStart
);

impl FromStr for StartEventContent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::StartEventContent>(json) {
            Ok(raw) => raw,
            Err(error) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: value,
                        message: error.to_string(),
                    }));
                }
                Err(error) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Deserialization { error }));
                }
            },
        };

        match raw {
            raw::StartEventContent::MSasV1(content) => {
                if !content
                    .key_agreement_protocols
                    .contains(&KeyAgreementProtocol::Curve25519)
                {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: serde_json::from_str::<Value>(json)?,
                        message: "`key_agreement_protocols` must contain at least `KeyAgreementProtocol::Curve25519`".to_string(),
                    }));
                }

                if !content.hashes.contains(&HashAlgorithm::Sha256) {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: serde_json::from_str::<Value>(json)?,
                        message: "`hashes` must contain at least `HashAlgorithm::Sha256`"
                            .to_string(),
                    }));
                }

                if !content
                    .message_authentication_codes
                    .contains(&MessageAuthenticationCode::HkdfHmacSha256)
                {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: serde_json::from_str::<Value>(json)?,
                        message: "`message_authentication_codes` must contain at least `MessageAuthenticationCode::HkdfHmacSha256`".to_string(),
                    }));
                }

                if !content
                    .short_authentication_string
                    .contains(&ShortAuthenticationString::Decimal)
                {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: serde_json::from_str::<Value>(json)?,
                        message: "`short_authentication_string` must contain at least `ShortAuthenticationString::Decimal`".to_string(),
                    }));
                }

                Ok(StartEventContent::MSasV1(content))
            }
            raw::StartEventContent::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.");
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for StartEventContent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for StartEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            StartEventContent::MSasV1(ref content) => content.serialize(serializer),
            _ => panic!("Attempted to serialize __Nonexhaustive variant."),
        }
    }
}

mod raw {
    use super::*;

    /// Begins an SAS key verification process.
    ///
    /// Typically sent as a to-device event.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct StartEvent {
        /// The event's content.
        pub content: StartEventContent,
    }

    /// The payload of an *m.key.verification.start* event.
    #[derive(Clone, Debug, PartialEq)]
    pub enum StartEventContent {
        /// The *m.sas.v1* verification method.
        MSasV1(MSasV1Content),

        /// Additional variants may be added in the future and will not be considered breaking changes
        /// to ruma-events.
        #[doc(hidden)]
        __Nonexhaustive,
    }

    impl<'de> Deserialize<'de> for StartEventContent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value: Value = Deserialize::deserialize(deserializer)?;

            let method_value = match value.get("method") {
                Some(value) => value.clone(),
                None => return Err(D::Error::missing_field("method")),
            };

            let method = match from_value::<VerificationMethod>(method_value.clone()) {
                Ok(method) => method,
                Err(error) => return Err(D::Error::custom(error.to_string())),
            };

            match method {
                VerificationMethod::MSasV1 => {
                    let content = match from_value::<MSasV1Content>(value) {
                        Ok(content) => content,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(StartEventContent::MSasV1(content))
                }
                VerificationMethod::__Nonexhaustive => Err(D::Error::custom(
                    "Attempted to deserialize __Nonexhaustive variant.",
                )),
            }
        }
    }
}

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
#[derive(Clone, Debug, PartialEq, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Deserialize)]
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
        if !options
            .key_agreement_protocols
            .contains(&KeyAgreementProtocol::Curve25519)
        {
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

        if !options
            .short_authentication_string
            .contains(&ShortAuthenticationString::Decimal)
        {
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

impl Serialize for MSasV1Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MSasV1Content", 2)?;

        state.serialize_field("from_device", &self.from_device)?;
        state.serialize_field("transaction_id", &self.transaction_id)?;
        state.serialize_field("method", "m.sas.v1")?;
        state.serialize_field("key_agreement_protocols", &self.key_agreement_protocols)?;
        state.serialize_field("hashes", &self.hashes)?;
        state.serialize_field(
            "message_authentication_codes",
            &self.message_authentication_codes,
        )?;
        state.serialize_field(
            "short_authentication_string",
            &self.short_authentication_string,
        )?;

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use super::{
        HashAlgorithm, KeyAgreementProtocol, MSasV1Content, MSasV1ContentOptions,
        MessageAuthenticationCode, ShortAuthenticationString, StartEvent, StartEventContent,
    };

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

        let key_verification_start = StartEvent {
            content: key_verification_start_content,
        };

        assert_eq!(
            to_string(&key_verification_start).unwrap(),
            r#"{"content":{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":["sha256"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]},"type":"m.key.verification.start"}"#
        );
    }

    #[test]
    fn deserialization() {
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

        // Deserialize the content struct separately to verify `FromStr` is implemented for it.
        assert_eq!(
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","hashes":["sha256"],"key_agreement_protocols":["curve25519"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]}"#
                .parse::<StartEventContent>()
                .unwrap(),
            key_verification_start_content
        );

        let key_verification_start = StartEvent {
            content: key_verification_start_content,
        };

        assert_eq!(
            r#"{"content":{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":["sha256"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]},"type":"m.key.verification.start"}"#
                .parse::<StartEvent>()
                .unwrap(),
            key_verification_start
        )
    }

    #[test]
    fn deserialization_failure() {
        // Invalid JSON
        let error = "{".parse::<StartEventContent>().err().unwrap();

        // No `serde_json::Value` available if deserialization failed.
        assert!(error.json().is_none());
    }

    #[test]
    fn deserialization_structure_mismatch() {
        // Missing several required fields.
        let error = r#"{"from_device":"123"}"#.parse::<StartEventContent>().err().unwrap();

        assert!(error.message().contains("missing field"));
        assert!(error.json().is_some());
    }

    #[test]
    fn deserialization_validation_missing_required_key_agreement_protocols() {
        let error =
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":[],"hashes":["sha256"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]}"#
                .parse::<StartEventContent>()
                .err()
                .unwrap();

        assert!(error.message().contains("key_agreement_protocols"));
        assert!(error.json().is_some());
    }

    #[test]
    fn deserialization_validation_missing_required_hashes() {
        let error =
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":[],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]}"#
                .parse::<StartEventContent>()
                .err()
                .unwrap();

        assert!(error.message().contains("hashes"));
        assert!(error.json().is_some());
    }

    #[test]
    fn deserialization_validation_missing_required_message_authentication_codes() {
        let error =
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":["sha256"],"message_authentication_codes":[],"short_authentication_string":["decimal"]}"#
                .parse::<StartEventContent>()
                .err()
                .unwrap();

        assert!(error.message().contains("message_authentication_codes"));
        assert!(error.json().is_some());
    }

    #[test]
    fn deserialization_validation_missing_required_short_authentication_string() {
        let error =
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":["sha256"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":[]}"#
                .parse::<StartEventContent>()
                .err()
                .unwrap();

        assert!(error.message().contains("short_authentication_string"));
        assert!(error.json().is_some());
    }
}
