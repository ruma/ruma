//! Endpoints for sending events.

/// [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod send_state_event_for_empty_key {
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: EventType
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub event_id: EventId,
    }


    impl ::Endpoint for Endpoint {
        type BodyParams = ::serde_json::Value;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/state/{}",
                params.room_id,
                params.event_type
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/rooms/:room_id/state/:event_type"
        }

        fn name() -> &'static str {
            "send_state_event_for_empty_key"
        }

        fn description() -> &'static str {
            "Send a state event to a room associated with the empty state key."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)
pub mod send_state_event_for_key {
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: EventType,
        pub state_key: String
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub event_id: EventId,
    }


    impl ::Endpoint for Endpoint {
        type BodyParams = ::serde_json::Value;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/state/{}/{}",
                params.room_id,
                params.event_type,
                params.state_key
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key"
        }

        fn name() -> &'static str {
            "send_state_event_for_key"
        }

        fn description() -> &'static str {
            "Send a state event to a room associated with a given state key."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)
pub mod send_message_event {
    use ruma_identifiers::{RoomId, EventId};
    use ruma_events::EventType;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: EventType,
        pub txn_id: String
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub event_id: EventId,
    }


    impl ::Endpoint for Endpoint {
        type BodyParams = ::serde_json::Value;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/send/{}/{}",
                params.room_id,
                params.event_type,
                params.txn_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id"
        }

        fn name() -> &'static str {
            "send_message_event"
        }

        fn description() -> &'static str {
            "Send a message event to a room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
