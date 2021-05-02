//! [POST /_matrix/federation/v1/get_missing_events/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#post-matrix-federation-v1-get-missing-events-roomid)

use js_int::{uint, UInt};
use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Retrieves previous events that the sender is missing.",
        method: POST,
        name: "get_missing_events",
        path: "/_matrix/federation/v1/get_missing_events/:room_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID to search in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The maximum number of events to retrieve. Defaults to 10.
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// The minimum depth of events to retrieve. Defaults to 0.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        pub min_depth: UInt,

        /// The latest event IDs that the sender already has. These are skipped when retrieving the previous events of `latest_events`.
        pub earliest_events: &'a [EventId],

        /// The event IDs to retrieve the previous events for.
        pub latest_events: &'a [EventId],
    }

    #[derive(Default)]
    response: {
        /// The missing PDUs.
        pub events: Vec<Raw<Pdu>>
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` for events in the given room with the given constraints.
    pub fn new(
        room_id: &'a RoomId,
        earliest_events: &'a [EventId],
        latest_events: &'a [EventId],
    ) -> Self {
        Self {
            room_id,
            limit: default_limit(),
            min_depth: UInt::default(),
            earliest_events,
            latest_events,
        }
    }
}

impl Response {
    /// Creates a new `Response` with the given events.
    pub fn new(events: Vec<Raw<Pdu>>) -> Self {
        Self { events }
    }
}

fn default_limit() -> UInt {
    uint!(10)
}

fn is_default_limit(val: &UInt) -> bool {
    *val == default_limit()
}
