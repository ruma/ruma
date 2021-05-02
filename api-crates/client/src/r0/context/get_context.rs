//! [GET /_matrix/client/r0/rooms/{roomId}/context/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-context-eventid)

use js_int::{uint, UInt};
use ruma_api::ruma_api;
use ruma_events::{AnyRoomEvent, AnyStateEvent};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

use crate::r0::filter::{IncomingRoomEventFilter, RoomEventFilter};

ruma_api! {
    metadata: {
        description: "Get the events immediately preceding and following a given event.",
        method: GET,
        path: "/_matrix/client/r0/rooms/:room_id/context/:event_id",
        name: "get_context",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to get events from.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event to get context around.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The maximum number of events to return.
        ///
        /// Defaults to 10.
        #[ruma_api(query)]
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// A RoomEventFilter to filter returned events with.
        #[ruma_api(query)]
        #[serde(
            with = "ruma_serde::json_string",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub filter: Option<RoomEventFilter<'a>>,
    }

    #[derive(Default)]
    response: {
        /// A token that can be used to paginate backwards with.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,

        /// A token that can be used to paginate forwards with.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,

        /// A list of room events that happened just before the requested event,
        /// in reverse-chronological order.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events_before: Vec<Raw<AnyRoomEvent>>,

        /// Details of the requested event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub event: Option<Raw<AnyRoomEvent>>,

        /// A list of room events that happened just after the requested event,
        /// in chronological order.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events_after: Vec<Raw<AnyRoomEvent>>,

        /// The state of the room at the last event returned.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub state: Vec<Raw<AnyStateEvent>>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id and event id.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId) -> Self {
        Self { room_id, event_id, limit: default_limit(), filter: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Default::default()
    }
}

fn default_limit() -> UInt {
    uint!(10)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_limit(val: &UInt) -> bool {
    *val == default_limit()
}
