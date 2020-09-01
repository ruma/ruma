//! [GET /_matrix/push/v1/notify](https://matrix.org/docs/spec/push_gateway/r0.1.1#post-matrix-push-v1-notify)

use std::time::SystemTime;

use js_int::{Int, UInt};
use ruma_api::ruma_api;
use ruma_common::{
    push::{PusherData, Tweak},
    Raw,
};
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

ruma_api! {
    metadata: {
        description: "Notify a push gateway about an event or update the number of unread notifications a user has",
        name: "push",
        method: POST,
        path: "/_matrix/push/v1/notify",
        rate_limited: false,
        requires_authentication: false,
    }

    #[non_exhaustive]
    request: {}

    #[non_exhaustive]
    response: {
        /// Information about the push notification.
        pub notification: Notification,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Notification {
    /// The event ID of the event being notified about..
    pub event_id: EventId,

    /// The room ID in which the event occurred.
    pub room_id: RoomId,

    /// The value `m.room.member`.
    #[serde(rename = "type")]
    pub kind: Option<EventType>,

    /// The matrix ID of the user who sent the original `m.room.third_party_invite`.
    pub sender: UserId,

    /// The current display name of the sender in the room in which the event occurred.
    pub sender_display_name: Option<String>,

    /// The name of the room in which the event occurred.
    pub room_name: Option<String>,

    /// An alias to display for the room in which the event occurred.
    pub room_alias: RoomAliasId,

    pub user_is_target: bool,

    pub prio: Priority,

    pub content: JsonValue,

    pub counts: Counts,

    pub devices: Vec<Device>,

    /// A timestamp added by the inviting homeserver.
    pub origin_server_ts: UInt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Counts {
    pub unread: Int,
    pub missed_calls: Int,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Device {
    /// The app_id given when the pusher was created.
    pub app_id: String,

    /// The pushkey given when the pusher was created.
    pub pushkey: String,

    /// The unix timestamp (in seconds) when the pushkey was last updated.
    pub pushkey_ts: SystemTime,

    /// A dictionary of additional pusher-specific data. For 'http' pushers, this is the data
    /// dictionary passed in at pusher creation minus the url key.
    pub data: PusherData,

    /// A dictionary of customizations made to the way this notification is to be presented.
    /// These are added by push rules.
    pub tweaks: Tweak,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Priority {
    High,
    Low,
}

impl Response {
    /// Creates a new `Response` with the given auth chain.
    pub fn new(notification: Notification) -> Self {
        Self { notification }
    }
}
