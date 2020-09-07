//! [POST /_matrix/push/v1/notify](https://matrix.org/docs/spec/push_gateway/r0.1.1#post-matrix-push-v1-notify)

use js_int::UInt;
use ruma_api::{ruma_api, Outgoing};
use ruma_client_api::r0::push::PusherData;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::BTreeMap, time::SystemTime};

ruma_api! {
    metadata: {
        description: "Notify a push gateway about an event or update the number of unread notifications a user has",
        name: "send_event_notification",
        method: POST,
        path: "/_matrix/push/v1/notify",
        rate_limited: false,
        requires_authentication: false,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {
        /// Information about the push notification
        notification: Notification<'a>,
    }

    #[derive(Default)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {
        /// A list of all pushkeys given in the notification request that are
        /// not valid.
        ///
        /// These could have been rejected by an upstream gateway because they
        /// have expired or have never been valid. Homeservers must cease
        /// sending notification requests for these pushkeys and remove the
        /// associated pushers. It may not necessarily be the notification in
        /// the request that failed: it could be that a previous notification to
        /// the same pushkey failed. May be empty.
        rejected: Vec<String>
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given notification.
    pub fn new(notification: Notification<'a>) -> Self {
        Self { notification }
    }
}

impl Response {
    /// Creates a new `Response` with the given list of rejected pushkeys.
    pub fn new(rejected: Vec<String>) -> Self {
        Self { rejected }
    }
}

/// Type for passing information about a push notification
#[derive(Clone, Debug, Outgoing, Serialize)]
pub struct Notification<'a> {
    /// The Matrix event ID of the event being notified about.
    ///
    /// This is required if the notification is about a particular Matrix event.
    /// It may be omitted for notifications that only contain updated badge
    /// counts. This ID can and should be used to detect duplicate notification
    /// requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<&'a EventId>,

    /// The ID of the room in which this event occurred.
    ///
    /// Required if the notification relates to a specific Matrix event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<&'a RoomId>,

    /// The type of the event as in the event's `type` field.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<&'a EventType>,

    /// The sender of the event as in the corresponding event field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<&'a UserId>,

    /// The current display name of the sender in the room in which the event
    /// occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_display_name: Option<&'a str>,

    /// The name of the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<&'a str>,

    /// An alias to display for the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_alias: Option<&'a RoomAliasId>,

    /// This is `true` if the user receiving the notification is the subject of
    /// a member event (i.e. the `state_key` of the member event is equal to the
    /// user's Matrix ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_is_target: Option<bool>,

    /// The priority of the notification.
    ///
    /// If omitted, `high` is assumed. This may be used by push gateways to
    /// deliver less time-sensitive notifications in a way that will preserve
    /// battery power on mobile devices. One of: ["high", "low"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prio: Option<NotificationPriority>,

    /// The `content` field from the event, if present. The pusher may omit this
    /// if the event had no content or for any other reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<&'a JsonValue>,

    /// This is a dictionary of the current number of unacknowledged
    /// communications for the recipient user. Counts whose value is zero should
    /// be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<NotificationCounts>,

    /// This is an array of devices that the notification should be sent to.
    pub devices: &'a [Device],
}

/// Type for passing information about notification priority.
///
/// This may be used by push gateways to deliver less time-sensitive
/// notifications in a way that will preserve battery power on mobile devices.
#[derive(Clone, Debug, Deserialize, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotificationPriority {
    /// A high priority notification
    High,
    /// A low priority notification
    Low,
}

/// Type for passing information about notification counts.
#[derive(Clone, Debug, Deserialize, Outgoing, Serialize)]
pub struct NotificationCounts {
    /// The number of unread messages a user has across all of the rooms they
    /// are a member of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread: Option<UInt>,

    /// The number of unacknowledged missed calls a user has across all rooms of
    /// which they are a member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missed_calls: Option<UInt>,
}

/// Type for passing information about devices.
#[derive(Clone, Debug, Deserialize, Outgoing, Serialize)]
pub struct Device {
    /// The `app_id` given when the pusher was created.
    pub app_id: String,

    /// The `pushkey` given when the pusher was created.
    pub pushkey: String,

    /// The unix timestamp (in seconds) when the pushkey was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushkey_ts: Option<SystemTime>,

    /// A dictionary of additional pusher-specific data. For 'http' pushers,
    /// this is the data dictionary passed in at pusher creation minus the `url`
    /// key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<PusherData>,

    /// A dictionary of customisations made to the way this notification is to
    /// be presented. These are added by push rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tweaks: Option<BTreeMap<String, String>>,
}
