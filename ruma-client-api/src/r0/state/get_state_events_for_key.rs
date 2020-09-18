//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::RoomId;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Get state events associated with a given key.",
        method: GET,
        name: "get_state_events_for_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
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

        /// The key of the state to look up.
        #[ruma_api(path)]
        pub state_key: &'a str,
    }

    response: {
        /// The content of the state event.
        #[ruma_api(body)]
        pub content: Box<RawJsonValue>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID, event type and state key.
    pub fn new(room_id: &'a RoomId, event_type: EventType, state_key: &'a str) -> Self {
        Self { room_id, event_type, state_key }
    }
}

impl Response {
    /// Creates a new `Response` with the given content.
    pub fn new(content: Box<RawJsonValue>) -> Self {
        Self { content }
    }
}
