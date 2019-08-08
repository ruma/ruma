//! [GET /_matrix/client/r0/rooms/{roomId}/context/{eventId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-context-eventid)

use ruma_api::ruma_api;
use ruma_events::{collections::only, EventResult};
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata {
        description: "Get the events immediately preceding and following a given event.",
        method: GET,
        path: "/_matrix/client/r0/rooms/:room_id/context/:event_id",
        name: "get_context",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The event to get context around.
        #[ruma_api(path)]
        pub event_id: EventId,
        /// The maximum number of events to return.
        ///
        /// Defaults to 10 if not supplied.
        #[ruma_api(query)]
        pub limit: u8,
        /// The room to get events from.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {
        /// A token that can be used to paginate forwards with.
        pub end: String,
        /// Details of the requested event.
        pub event: EventResult<only::RoomEvent>,
        /// A list of room events that happened just after the requested event, in chronological
        /// order.
        pub events_after: Vec<EventResult<only::RoomEvent>>,
        /// A list of room events that happened just before the requested event, in
        /// reverse-chronological order.
        pub events_before: Vec<EventResult<only::RoomEvent>>,
        /// A token that can be used to paginate backwards with.
        pub start: String,
        /// The state of the room at the last event returned.
        pub state: Vec<EventResult<only::StateEvent>>,
    }
}
