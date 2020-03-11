//! Types for the *m.room_key_request* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::{DeviceId, RoomId};
use serde::{Deserialize, Serialize};

use super::Algorithm;

ruma_event! {
    /// This event type is used to request keys for end-to-end encryption.
    ///
    /// It is sent as an unencrypted to-device event.
    RoomKeyRequestEvent {
        kind: Event,
        event_type: RoomKeyRequest,
        content: {
            /// Whether this is a new key request or a cancellation of a previous request.
            pub action: Action,

            /// Information about the requested key.
            ///
            /// Required when action is `request`.
            pub body: Option<RequestedKeyInfo>,

            /// ID of the device requesting the key.
            pub requesting_device_id: DeviceId,

            /// A random string uniquely identifying the request for a key.
            ///
            /// If the key is requested multiple times, it should be reused. It should also reused
            /// in order to cancel a request.
            pub request_id: String,
        },
    }
}

/// A new key request or a cancellation of a previous request.
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Serialize)]
pub enum Action {
    /// Request a key.
    #[serde(rename = "request")]
    Request,

    /// Cancel a request for a key.
    #[serde(rename = "request_cancellation")]
    CancelRequest,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    Action {
        Request => "request",
        CancelRequest => "cancel_request",
    }
}

/// Information about a requested key.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RequestedKeyInfo {
    /// The encryption algorithm the requested key in this event is to be used with.
    pub algorithm: Algorithm,

    /// The room where the key is used.
    pub room_id: RoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,
}
