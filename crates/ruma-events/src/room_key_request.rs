//! Types for the [`m.room_key_request`] event.
//!
//! [`m.room_key_request`]: https://spec.matrix.org/latest/client-server-api/#mroom_key_request

use ruma_common::{
    serde::StringEnum, EventEncryptionAlgorithm, OwnedDeviceId, OwnedRoomId, OwnedTransactionId,
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The content of an `m.room_key_request` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room_key_request", kind = ToDevice)]
pub struct ToDeviceRoomKeyRequestEventContent {
    /// Whether this is a new key request or a cancellation of a previous request.
    pub action: Action,

    /// Information about the requested key.
    ///
    /// Required if action is `request`.
    pub body: Option<RequestedKeyInfo>,

    /// ID of the device requesting the key.
    pub requesting_device_id: OwnedDeviceId,

    /// A random string uniquely identifying the request for a key.
    ///
    /// If the key is requested multiple times, it should be reused. It should also reused
    /// in order to cancel a request.
    pub request_id: OwnedTransactionId,
}

impl ToDeviceRoomKeyRequestEventContent {
    /// Creates a new `ToDeviceRoomKeyRequestEventContent` with the given action, boyd, device ID
    /// and request ID.
    pub fn new(
        action: Action,
        body: Option<RequestedKeyInfo>,
        requesting_device_id: OwnedDeviceId,
        request_id: OwnedTransactionId,
    ) -> Self {
        Self { action, body, requesting_device_id, request_id }
    }
}

/// A new key request or a cancellation of a previous request.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Action {
    /// Request a key.
    Request,

    /// Cancel a request for a key.
    #[ruma_enum(rename = "request_cancellation")]
    CancelRequest,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Information about a requested key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RequestedKeyInfo {
    /// The encryption algorithm the requested key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    #[deprecated = "this field still needs to be sent but should not be used when received"]
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,
}

impl RequestedKeyInfo {
    /// Creates a new `RequestedKeyInfo` with the given algorithm, room ID, sender key and session
    /// ID.
    pub fn new(
        algorithm: EventEncryptionAlgorithm,
        room_id: OwnedRoomId,
        sender_key: String,
        session_id: String,
    ) -> Self {
        #[allow(deprecated)]
        Self { algorithm, room_id, sender_key, session_id }
    }
}
