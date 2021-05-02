//! Types for the *m.room_key_request* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::{DeviceIdBox, EventEncryptionAlgorithm, RoomId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

/// The payload for `RoomKeyRequestEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.room_key_request")]
pub struct RoomKeyRequestToDeviceEventContent {
    /// Whether this is a new key request or a cancellation of a previous request.
    pub action: Action,

    /// Information about the requested key.
    ///
    /// Required when action is `request`.
    pub body: Option<RequestedKeyInfo>,

    /// ID of the device requesting the key.
    pub requesting_device_id: DeviceIdBox,

    /// A random string uniquely identifying the request for a key.
    ///
    /// If the key is requested multiple times, it should be reused. It should also reused
    /// in order to cancel a request.
    pub request_id: String,
}

/// A new key request or a cancellation of a previous request.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum Action {
    /// Request a key.
    Request,

    /// Cancel a request for a key.
    #[ruma_enum(rename = "request_cancellation")]
    CancelRequest,

    #[doc(hidden)]
    _Custom(String),
}

/// Information about a requested key.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
