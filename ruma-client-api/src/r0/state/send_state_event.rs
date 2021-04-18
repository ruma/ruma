//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::{AnyStateEventContent, EventContent as _};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Send a state event to a room associated with a given state key.",
        method: PUT,
        name: "send_state_event",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to set the state in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: &'a str,

        /// The state_key for the state to send.
        #[ruma_api(path)]
        pub state_key: &'a str,

        /// The event content to send.
        #[ruma_api(body)]
        pub body: Raw<AnyStateEventContent>,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id, state key and event content.
    pub fn new(room_id: &'a RoomId, state_key: &'a str, content: &'a AnyStateEventContent) -> Self {
        Self { room_id, event_type: content.event_type(), body: content.into(), state_key }
    }

    /// Creates a new `Request` with the given room id, event type, state key and raw event content.
    pub fn new_raw(
        room_id: &'a RoomId,
        event_type: &'a str,
        state_key: &'a str,
        body: Raw<AnyStateEventContent>,
    ) -> Self {
        Self { room_id, event_type, state_key, body }
    }
}

impl Response {
    /// Creates a new `Response` with the given event id.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}
