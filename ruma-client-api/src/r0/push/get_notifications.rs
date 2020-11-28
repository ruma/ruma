//! [GET /_matrix/client/r0/notifications](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-notifications)

use std::time::SystemTime;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::push::Action;
use ruma_events::AnyEvent;
use ruma_identifiers::RoomId;
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Paginate through the list of events that the user has been, or would have been notified about.",
        method: GET,
        name: "get_notifications",
        path: "/_matrix/client/r0/notifications",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// Pagination token given to retrieve the next set of events.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<&'a str>,

        /// Limit on the number of events to return in this request.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Allows basic filtering of events returned. Supply "highlight" to return only events
        /// where the notification had the 'highlight' tweak set.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub only: Option<&'a str>
    }

    response: {
        /// The token to supply in the from param of the next /notifications request in order
        /// to request more events. If this is absent, there are no more results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_token: Option<String>,


        /// The list of events that triggered notifications.
        pub notifications: Vec<Notification>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given notifications.
    pub fn new(notifications: Vec<Notification>) -> Self {
        Self { next_token: None, notifications }
    }
}

/// Represents a notification.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Notification {
    /// The actions to perform when the conditions for this rule are met.
    pub actions: Vec<Action>,

    /// The event that triggered the notification.
    pub event: Raw<AnyEvent>,

    /// The profile tag of the rule that matched this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// Indicates whether the user has sent a read receipt indicating that they have read this
    /// message.
    pub read: bool,

    /// The ID of the room in which the event was posted.
    pub room_id: RoomId,

    /// The time at which the event notification was sent.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub ts: SystemTime,
}

impl Notification {
    /// Creates a new `Notification` with the given actions, event, read flag, room ID and
    /// timestamp.
    pub fn new(
        actions: Vec<Action>,
        event: Raw<AnyEvent>,
        read: bool,
        room_id: RoomId,
        ts: SystemTime,
    ) -> Self {
        Self { actions, event, profile_tag: None, read, room_id, ts }
    }
}
