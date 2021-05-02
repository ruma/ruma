//! [GET /_matrix/client/r0/rooms/{roomId}/event/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-event-eventid)

use ruma_api::ruma_api;
use ruma_events::AnyRoomEvent;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Get a single event based on roomId/eventId",
        method: GET,
        name: "get_room_event",
        path: "/_matrix/client/r0/rooms/:room_id/event/:event_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The ID of the room the event is in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The ID of the event.
        #[ruma_api(path)]
        pub event_id: &'a EventId,
    }

    response: {
        /// Arbitrary JSON of the event body.
        #[ruma_api(body)]
        pub event: Raw<AnyRoomEvent>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID and event ID.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId) -> Self {
        Self { room_id, event_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given event.
    pub fn new(event: Raw<AnyRoomEvent>) -> Self {
        Self { event }
    }
}
