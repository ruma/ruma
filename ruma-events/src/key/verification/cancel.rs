//! Types for the *m.key.verification.cancel* event.

use ruma_events_macros::BasicEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events_macros::MessageEventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Cancels a key verification process/request.
#[cfg(feature = "unstable-pre-spec")]
pub type CancelEvent = MessageEvent<CancelEventContent>;

/// The payload for a to-device `CancelEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.cancel")]
pub struct CancelToDeviceEventContent {
    /// The opaque identifier for the verification process/request.
    pub transaction_id: String,

    /// A human readable description of the `code`.
    ///
    /// The client should only rely on this string if it does not understand the `code`.
    pub reason: String,

    /// The error code for why the process/request was cancelled by the user.
    pub code: CancelCode,
}

/// The payload for an in-room `CancelEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.cancel")]
#[cfg(feature = "unstable-pre-spec")]
pub struct CancelEventContent {
    /// A human readable description of the `code`.
    ///
    /// The client should only rely on this string if it does not understand the `code`.
    pub reason: String,

    /// The error code for why the process/request was cancelled by the user.
    pub code: CancelCode,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

/// An error code for why the process/request was cancelled by the user.
///
/// Custom error codes should use the Java package naming convention.
///
/// This type can hold an arbitrary string. To check for events that are not
/// available as a documented variant here, use its string representation,
/// obtained through `.as_str()`.
// FIXME: Add `m.foo_bar` as a naming scheme in StringEnum and remove rename attributes.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum CancelCode {
    /// The user cancelled the verification.
    #[ruma_enum(rename = "m.user")]
    User,

    /// The verification process timed out. Verification processes can define their own timeout
    /// parameters.
    #[ruma_enum(rename = "m.timeout")]
    Timeout,

    /// The device does not know about the given transaction ID.
    #[ruma_enum(rename = "m.unknown_transaction")]
    UnknownTransaction,

    /// The device does not know how to handle the requested method.
    ///
    /// This should be sent for *m.key.verification.start* messages and messages defined by
    /// individual verification processes.
    #[ruma_enum(rename = "m.unknown_method")]
    UnknownMethod,

    /// The device received an unexpected message.
    ///
    /// Typically raised when one of the parties is handling the verification out of order.
    #[ruma_enum(rename = "m.unexpected_message")]
    UnexpectedMessage,

    /// The key was not verified.
    #[ruma_enum(rename = "m.key_mismatch")]
    KeyMismatch,

    /// The expected user did not match the user verified.
    #[ruma_enum(rename = "m.user_mismatch")]
    UserMismatch,

    /// The message received was invalid.
    #[ruma_enum(rename = "m.invalid_message")]
    InvalidMessage,

    /// An *m.key.verification.request* was accepted by a different device.
    ///
    /// The device receiving this error can ignore the verification request.
    #[ruma_enum(rename = "m.accepted")]
    Accepted,

    #[doc(hidden)]
    _Custom(String),
}

impl CancelCode {
    /// Creates a string slice from this `CancelCode`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
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
