//! Types for the *m.key.verification.cancel* event.

use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{
    de::{Error as SerdeError, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

event! {
    /// Cancels a key verification process/request.
    ///
    /// Typically sent as a to-device event.
    pub struct CancelEvent(CancelEventContent) {}
}

/// The payload of an *m.key.verification.cancel* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CancelEventContent {
    /// The opaque identifier for the verification process/request.
    pub transaction_id: String,

    /// A human readable description of the `code`.
    ///
    /// The client should only rely on this string if it does not understand the `code`.
    pub reason: String,

    /// The error code for why the process/request was cancelled by the user.
    pub code: CancelCode,
}

/// An error code for why the process/request was cancelled by the user.
///
/// Custom error codes should use the Java package naming convention.
#[derive(Clone, Debug, PartialEq)]
pub enum CancelCode {
    /// The user cancelled the verification.
    User,

    /// The verification process timed out. Verification processes can define their own timeout
    /// parameters.
    Timeout,

    /// The device does not know about the given transaction ID.
    UnknownTransaction,

    /// The device does not know how to handle the requested method.
    ///
    /// This should be sent for *m.key.verification.start* messages and messages defined by
    /// individual verification processes.
    UnknownMethod,

    /// The device received an unexpected message.
    ///
    /// Typically raised when one of the parties is handling the verification out of order.
    UnexpectedMessage,

    /// The key was not verified.
    KeyMismatch,

    /// The expected user did not match the user verified.
    UserMismatch,

    /// The message received was invalid.
    InvalidMessage,

    /// An *m.key.verification.request* was accepted by a different device.
    ///
    /// The device receiving this error can ignore the verification request.
    Accepted,

    /// Any code that is not part of the specification.
    Custom(String),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for CancelCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let cancel_code_str = match *self {
            CancelCode::User => "m.user",
            CancelCode::Timeout => "m.timeout",
            CancelCode::UnknownTransaction => "m.unknown_transaction",
            CancelCode::UnknownMethod => "m.unknown_method",
            CancelCode::UnexpectedMessage => "m.unexpected_message",
            CancelCode::KeyMismatch => "m.key_mismatch",
            CancelCode::UserMismatch => "m.user_mismatch",
            CancelCode::InvalidMessage => "m.invalid_message",
            CancelCode::Accepted => "m.accepted",
            CancelCode::Custom(ref cancel_code) => cancel_code,
            CancelCode::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
            }
        };

        write!(f, "{}", cancel_code_str)
    }
}

impl<'a> From<&'a str> for CancelCode {
    fn from(s: &'a str) -> CancelCode {
        match s {
            "m.user" => CancelCode::User,
            "m.timeout" => CancelCode::Timeout,
            "m.unknown_transaction" => CancelCode::UnknownTransaction,
            "m.unknown_method" => CancelCode::UnknownMethod,
            "m.unexpected_message" => CancelCode::UnexpectedMessage,
            "m.key_mismatch" => CancelCode::KeyMismatch,
            "m.user_mismatch" => CancelCode::UserMismatch,
            "m.invalid_message" => CancelCode::InvalidMessage,
            "m.accepted" => CancelCode::Accepted,
            cancel_code => CancelCode::Custom(cancel_code.to_string()),
        }
    }
}

impl Serialize for CancelCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CancelCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CancelCodeVisitor;

        impl<'de> Visitor<'de> for CancelCodeVisitor {
            type Value = CancelCode;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                write!(formatter, "an `m.key.verification.cancel` code as a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(CancelCode::from(v))
            }
        }

        deserializer.deserialize_str(CancelCodeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::CancelCode;

    #[test]
    fn cancel_codes_serialize_to_display_form() {
        assert_eq!(to_string(&CancelCode::User).unwrap(), r#""m.user""#);
    }

    #[test]
    fn custom_cancel_codes_serialize_to_display_form() {
        assert_eq!(
            to_string(&CancelCode::Custom("io.ruma.test".to_string())).unwrap(),
            r#""io.ruma.test""#
        );
    }

    #[test]
    fn cancel_codes_deserialize_from_display_form() {
        assert_eq!(
            from_str::<CancelCode>(r#""m.user""#).unwrap(),
            CancelCode::User
        );
    }

    #[test]
    fn custom_cancel_codes_deserialize_from_display_form() {
        assert_eq!(
            from_str::<CancelCode>(r#""io.ruma.test""#).unwrap(),
            CancelCode::Custom("io.ruma.test".to_string())
        )
    }
}
