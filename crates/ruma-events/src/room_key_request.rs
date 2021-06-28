//! Types for the *m.room_key_request* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::{DeviceIdBox, EventEncryptionAlgorithm, RoomId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

/// The payload for `RoomKeyRequestEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room_key_request", kind = ToDevice)]
pub struct RoomKeyRequestToDeviceEventContent {
    /// Whether this is a new key request or a cancellation of a previous request.
    pub action: Action,

    /// Information about the requested key.
    ///
    /// Required if action is `request`.
    pub body: Option<RequestedKeyInfo>,

    /// ID of the device requesting the key.
    pub requesting_device_id: DeviceIdBox,

    /// A random string uniquely identifying the request for a key.
    ///
    /// If the key is requested multiple times, it should be reused. It should also reused
    /// in order to cancel a request.
    pub request_id: String,
}

impl RoomKeyRequestToDeviceEventContent {
    /// Creates a new `RoomKeyRequestToDeviceEventContent` with the given action, boyd, device ID
    /// and request ID.
    pub fn new(
        action: Action,
        body: Option<RequestedKeyInfo>,
        requesting_device_id: DeviceIdBox,
        request_id: String,
    ) -> Self {
        Self { action, body, requesting_device_id, request_id }
    }
}

/// A new key request or a cancellation of a previous request.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Action {
    /// Request a key.
    Request,

    /// Cancel a request for a key.
    #[ruma_enum(rename = "request_cancellation")]
    CancelRequest,

    #[doc(hidden)]
    _Custom(String),
}

impl Action {
    /// Creates a string slice from this `Action`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// Information about a requested key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RequestedKeyInfo {
    /// The encryption algorithm the requested key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: RoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,
}

impl RequestedKeyInfo {
    /// Creates a new `RequestedKeyInfo` with the given algorithm, room ID, sender key and session
    /// ID.
    pub fn new(
        algorithm: EventEncryptionAlgorithm,
        room_id: RoomId,
        sender_key: String,
        session_id: String,
    ) -> Self {
        Self { algorithm, room_id, sender_key, session_id }
    }
}
