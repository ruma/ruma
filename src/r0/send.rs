//! Endpoints for sending events.

/// [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod send_state_event_for_empty_key {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;
    use serde_json::Value;

    ruma_api! {
        metadata {
            description: "Send a state event to a room associated with the empty state key.",
            method: PUT,
            name: "send_state_event_for_empty_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to set the state in.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of event to send.
            #[ruma_api(path)]
            pub event_type: EventType,
            /// The event's content.
            #[ruma_api(body)]
            pub data: Value,
        }

        response {
            /// A unique identifier for the event.
            pub event_id: EventId,
        }
    }
}

/// [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)
pub mod send_state_event_for_key {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;
    use serde_json::Value;

    ruma_api! {
        metadata {
            description: "Send a state event to a room associated with a given state key.",
            method: PUT,
            name: "send_state_event_for_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to set the state in.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of event to send.
            #[ruma_api(path)]
            pub event_type: EventType,
            /// The state_key for the state to send. Defaults to the empty string.
            #[ruma_api(path)]
            pub state_key: String,
            /// The event's content.
            #[ruma_api(body)]
            pub data: Value,
        }

        response {
            /// A unique identifier for the event.
            pub event_id: EventId,
        }
    }
}

/// [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)
pub mod send_message_event {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;
    use ruma_events::room::message::MessageEventContent;

    ruma_api! {
        metadata {
            description: "Send a message event to a room.",
            method: PUT,
            name: "send_message_event",
            path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to send the event to.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of event to send.
            #[ruma_api(path)]
            pub event_type: EventType,
            /// The transaction ID for this event.
            ///
            /// Clients should generate an ID unique across requests with the
            /// same access token; it will be used by the server to ensure
            /// idempotency of requests.
            #[ruma_api(path)]
            pub txn_id: String,
            /// The event's content.
            #[ruma_api(body)]
            pub data: MessageEventContent,
        }

        response {
            /// A unique identifier for the event.
            pub event_id: EventId,
        }
    }
}
