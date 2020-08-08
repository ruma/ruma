//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId};
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Send a state event to a room associated with a given state key.",
        method: PUT,
        name: "create_state_event_for_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The room to set the state in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: EventType,

        /// The state_key for the state to send. Defaults to the empty string.
        #[ruma_api(path)]
        pub state_key: &'a str,

        /// The event's content. The type for this field will be updated in a
        /// future release, until then you can create a value using
        /// `serde_json::value::to_raw_value`.
        #[ruma_api(body)]
        pub data: Box<RawJsonValue>,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}
