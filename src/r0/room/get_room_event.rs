//! [GET /_matrix/client/r0/rooms/{roomId}/event/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-event-eventid)

use ruma_api::ruma_api;
use ruma_events::{collections::all, EventResult};
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata {
        description: "Get a single event based on roomId/eventId",
        method: GET,
        name: "get_room_event",
        path: "/_matrix/client/r0/rooms/:room_id/event/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The ID of the room the event is in.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The ID of the event.
        #[ruma_api(path)]
        pub event_id: EventId,
    }

    response {
        /// Arbitrary JSON of the event body. Returns both room and state events.
        #[wrap_incoming(with EventResult)]
        pub event: all::RoomEvent,
    }

    error: crate::Error
}
