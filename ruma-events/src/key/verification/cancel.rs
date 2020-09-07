//! Types for the *m.key.verification.cancel* event.

use std::fmt::{Display, Formatter, Result as FmtResult};

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Cancels a key verification process/request.
///
/// Typically sent as a to-device event.
pub type CancelEvent = BasicEvent<CancelEventContent>;

/// The payload for `CancelEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.cancel")]
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
///
/// This type can hold an arbitrary string. To check for events that are not
/// available as a documented variant here, use its string representation,
/// obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(from = "String", into = "String")]
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

    #[doc(hidden)]
    _Custom(String),
}

impl CancelCode {
    /// Creates a string slice from this `CancelCode`.
    pub fn as_str(&self) -> &str {
        match *self {
            CancelCode::User => "m.user",
            CancelCode::Timeout => "m.timeout",
            CancelCode::UnknownTransaction => "m.unknown_transaction",
            CancelCode::UnknownMethod => "m.unknown_method",
            CancelCode::UnexpectedMessage => "m.unexpected_message",
            CancelCode::KeyMismatch => "m.key_mismatch",
            CancelCode::UserMismatch => "m.user_mismatch",
            CancelCode::InvalidMessage => "m.invalid_message",
            CancelCode::Accepted => "m.accepted",
            CancelCode::_Custom(ref cancel_code) => cancel_code,
        }
    }
}

impl Display for CancelCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_str())
    }
}

impl<T> From<T> for CancelCode
where
    T: Into<String> + AsRef<str>,
{
    fn from(s: T) -> CancelCode {
        match s.as_ref() {
            "m.user" => CancelCode::User,
            "m.timeout" => CancelCode::Timeout,
            "m.unknown_transaction" => CancelCode::UnknownTransaction,
            "m.unknown_method" => CancelCode::UnknownMethod,
            "m.unexpected_message" => CancelCode::UnexpectedMessage,
            "m.key_mismatch" => CancelCode::KeyMismatch,
            "m.user_mismatch" => CancelCode::UserMismatch,
            "m.invalid_message" => CancelCode::InvalidMessage,
            "m.accepted" => CancelCode::Accepted,
            _ => CancelCode::_Custom(s.into()),
        }
    }
}

impl From<CancelCode> for String {
    fn from(cancel_code: CancelCode) -> String {
        cancel_code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::CancelCode;

    #[test]
    fn cancel_codes_serialize_to_display_form() {
        assert_eq!(to_json_value(&CancelCode::User).unwrap(), json!("m.user"));
    }

    #[test]
    fn custom_cancel_codes_serialize_to_display_form() {
        assert_eq!(
            to_json_value(&CancelCode::_Custom("io.ruma.test".into())).unwrap(),
            json!("io.ruma.test")
        );
    }

    #[test]
    fn cancel_codes_deserialize_from_display_form() {
        assert_eq!(from_json_value::<CancelCode>(json!("m.user")).unwrap(), CancelCode::User);
    }

    #[test]
    fn custom_cancel_codes_deserialize_from_display_form() {
        assert_eq!(
            from_json_value::<CancelCode>(json!("io.ruma.test")).unwrap(),
            CancelCode::_Custom("io.ruma.test".into())
        )
    }
}
