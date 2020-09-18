//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-state-eventtype)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::RoomId;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Get state events of a given type associated with the empty key.",
        method: GET,
        name: "get_state_events_for_empty_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The type of state to look up.
        #[ruma_api(path)]
        pub event_type: EventType,
    }

    response: {
        /// The content of the state event.
        ///
        /// To create a `Box<RawJsonValue>`, use `serde_json::value::to_raw_value`.
        #[ruma_api(body)]
        pub content: Box<RawJsonValue>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID and event type.
    pub fn new(room_id: &'a RoomId, event_type: EventType) -> Self {
        Self { room_id, event_type }
    }
}

impl Response {
    /// Creates a new `Response` with the given content.
    pub fn new(content: Box<RawJsonValue>) -> Self {
        Self { content }
    }
}
