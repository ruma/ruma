//! Endpoints for getting events

/// GET /_matrix/client/r0/rooms/{roomId}/state
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state)
pub mod get_full_state {
    use ruma_identifiers::RoomId;
    use ruma_events::collections::only;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Vec<only::StateEvent>;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/state",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/state".to_string()
        }
    }
}

/// GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod get_state_for_empty_key {
    use ruma_identifiers::RoomId;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ::serde_json::Value;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/state/{}",
                params.room_id,
                params.event_type
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/state/:event_type".to_string()
        }
    }
}

/// GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)
pub mod get_state_for_key {
    use ruma_identifiers::RoomId;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: String,
        pub state_key: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ::serde_json::Value;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/state/{}/{}",
                params.room_id,
                params.event_type,
                params.state_key
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key".to_string()
        }
    }
}

/// GET /_matrix/client/r0/rooms/{roomId}/members
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-members)
pub mod get_members {
    use ruma_identifiers::RoomId;
    use ruma_events::room::member::MemberEvent;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: String
    }
    
    /// This API endpoint's reponse.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub chunks: Vec<MemberEvent>
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/members",
                params.room_id,
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/members".to_string()
        }
    }
}

/// GET /_matrix/client/r0/rooms/{roomId}/messages
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-messages)
pub mod get_messages {
    use ruma_identifiers::RoomId;
    use ruma_events::collections::only;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
        pub event_type: String
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum Direction {
        #[serde(rename="b")]
        Backward,
        #[serde(rename="f")]
        Forward
    }

    /// This API endpoint's query parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        pub from: String,
        pub to: Option<String>,
        pub dir: Direction,
        pub limit: Option<u64>
    }
    
    /// This API endpoint's reponse.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub start: String,
        pub chunks: Vec<only::RoomEvent>,
        pub end: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = QueryParams;
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/messages",
                params.room_id,
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/messages".to_string()
        }
    }
}
