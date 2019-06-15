//! Types for the *m.key.verification.start* event.

use ruma_identifiers::DeviceId;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
    VerificationMethod,
};

event! {
    /// Begins an SAS key verification process.
    ///
    /// Typically sent as a to-device event.
    pub struct StartEvent(StartEventContent) {}
}

/// The payload of an *m.key.verification.start* event.
#[derive(Clone, Debug, PartialEq)]
pub enum StartEventContent {
    /// The *m.sas.v1* verification method.
    MSasV1(MSasV1Content),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to `ruma-events`.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// The payload of an *m.key.verification.start* event using the *m.sas.v1* method.
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct MSasV1Content {
    /// The device ID which is initiating the process.
    pub from_device: DeviceId,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the *m.key.verification.request* if this process is originating
    /// from a request.
    pub transaction_id: String,

    /// The verification method to use.
    ///
    /// Must be `m.sas.v1`.
    pub method: VerificationMethod,

    /// Optional method to use to verify the other user's key with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_method: Option<VerificationMethod>,

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

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{
        HashAlgorithm, KeyAgreementProtocol, MSasV1Content, MessageAuthenticationCode,
        ShortAuthenticationString, StartEventContent, VerificationMethod,
    };

    #[test]
    fn serializtion() {
        let key_verification_start_content = StartEventContent::MSasV1(MSasV1Content {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
            method: VerificationMethod::MSasV1,
            next_method: None,
            hashes: vec![HashAlgorithm::Sha256],
            key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
            message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
            short_authentication_string: vec![ShortAuthenticationString::Decimal],
        });

        assert_eq!(
            to_string(&key_verification_start_content).unwrap(),
            r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","key_agreement_protocols":["curve25519"],"hashes":["sha256"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]}"#
        );
    }

    #[test]
    fn deserialization() {
        let key_verification_start_content = StartEventContent::MSasV1(MSasV1Content {
            from_device: "123".to_string(),
            transaction_id: "456".to_string(),
            method: VerificationMethod::MSasV1,
            next_method: None,
            hashes: vec![HashAlgorithm::Sha256],
            key_agreement_protocols: vec![KeyAgreementProtocol::Curve25519],
            message_authentication_codes: vec![MessageAuthenticationCode::HkdfHmacSha256],
            short_authentication_string: vec![ShortAuthenticationString::Decimal],
        });

        assert_eq!(
            from_str::<StartEventContent>(
                r#"{"from_device":"123","transaction_id":"456","method":"m.sas.v1","hashes":["sha256"],"key_agreement_protocols":["curve25519"],"message_authentication_codes":["hkdf-hmac-sha256"],"short_authentication_string":["decimal"]}"#
            )
            .unwrap(),
            key_verification_start_content
        );
    }

    #[test]
    fn deserialization_failure() {
        assert!(from_str::<StartEventContent>(r#"{"from_device":"123"}"#).is_err());
    }
}
