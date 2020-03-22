//! [GET /_matrix/client/r0/notifications](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-notifications)

use js_int::UInt;
use ruma_api::{ruma_api, Outgoing};
use ruma_events::{collections::all, EventResult};
use ruma_identifiers::RoomId;
use serde::Serialize;

use super::Action;

ruma_api! {
    metadata {
        description: "Paginate through the list of events that the user has been, or would have been notified about.",
        method: GET,
        name: "get_notifications",
        path: "/_matrix/client/r0/notifications",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Pagination token given to retrieve the next set of events.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

        /// Limit on the number of events to return in this request.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Allows basic filtering of events returned. Supply "highlight" to return only events where
        /// the notification had the 'highlight' tweak set.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub only: Option<String>
    }

    response {
        /// The token to supply in the from param of the next /notifications request in order
        /// to request more events. If this is absent, there are no more results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_token: Option<String>,


        /// The list of events that triggered notifications.
        #[wrap_incoming(Notification)]
        pub notifications: Vec<Notification>,
    }

    error: crate::Error
}

/// Represents a notification
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct Notification {
    /// The actions to perform when the conditions for this rule are met.
    pub actions: Vec<Action>,

    /// The event that triggered the notification.
    #[wrap_incoming(with EventResult)]
    pub event: all::Event,

    /// The profile tag of the rule that matched this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// Indicates whether the user has sent a read receipt indicating that they have read this message.
    pub read: bool,

    /// The ID of the room in which the event was posted.
    pub room_id: RoomId,

    /// The unix timestamp at which the event notification was sent, in milliseconds.
    pub ts: UInt,
}
