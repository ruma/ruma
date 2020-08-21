//! [POST /_matrix/federation/v1/get_missing_events/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#post-matrix-federation-v1-get-missing-events-roomid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata: {
        description: "Retrieves previous events that the sender is missing.",
        method: POST,
        name: "get_missing_events",
        path: "/_matrix/federation/v1/get_missing_events/:room_id",
        rate_limited: false,
        requires_authentication: true,
    }

    #[non_exhaustive]
    request: {
        /// The room ID to search in.
        #[ruma_api(path)]
        room_id: RoomId,

        /// The maximum number of events to retrieve. Defaults to 10.
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<UInt>,

        /// The minimum depth of events to retrieve. Defaults to 0.
        #[serde(skip_serializing_if = "Option::is_none")]
        min_depth: Option<UInt>,

        /// The latest event IDs that the sender already has. These are skipped when retrieving the previous events of `latest_events`.
        earliest_events: Vec<EventId>,

        /// The event IDs to retrieve the previous events for.
        latest_events: Vec<EventId>
    }

    response: {
        /// The missing events.
        events: Vec<Pdu>
    }
}

impl Request {
    /// Creates a new `Request` for events in the given room with the given constraints.
    pub fn new(
        room_id: RoomId,
        limit: Option<UInt>,
        min_depth: Option<UInt>,
        earliest_events: Vec<EventId>,
        latest_events: Vec<EventId>,
    ) -> Self {
        Self { room_id, limit, min_depth, earliest_events, latest_events }
    }
}

impl Response {
    /// Creates a new `Response` with the given events.
    pub fn new(events: Vec<Pdu>) -> Self {
        Self { events }
    }
}
