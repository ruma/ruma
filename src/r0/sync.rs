//! Endpoints for getting and synchronizing events.

/// [GET /_matrix/client/r0/rooms/{roomId}/state](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state)
pub mod get_state_events {
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

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod get_state_event_by_event_type {
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

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)
pub mod get_state_event_by_state_key {
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

/// [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-members)
pub mod get_member_events {
    use ruma_identifiers::RoomId;
    use ruma_events::room::member::MemberEvent;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
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

/// [GET /_matrix/client/r0/rooms/{roomId}/messages](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-messages)
pub mod get_message_events {
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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to: Option<String>,
        pub dir: Direction,
        #[serde(skip_serializing_if = "Option::is_none")]
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

/// [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-sync)
pub mod sync_events {
    use ruma_events::collections::only;
    use ruma_identifiers::RoomId;

    use std::collections::HashMap;

    use r0::filter::FilterDefinition;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// Whether to set presence or not during sync.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum SetPresence {
        #[serde(rename="offline")]
        Offline
    }

    /// A filter represented either as its full JSON definition or the ID of a saved filter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum Filter {
        /// A complete filter definition serialized to JSON.
        FilterDefinition(FilterDefinition),
        /// The ID of a filter saved on the server.
        FilterId(String),
    }

    /// This API endpoint's query parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filter: Option<Filter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub full_state: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub set_presence: Option<SetPresence>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub timeout: Option<u64>
    }

    /// Updates to rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Rooms {
        pub leave: HashMap<RoomId, LeftRoom>,
        pub join: HashMap<RoomId, JoinedRoom>,
        pub invite: HashMap<RoomId, InvitedRoom>
    }

    /// Historical updates to left rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LeftRoom {
        pub timeline: Timeline,
        pub state: State
    }

    /// Updates to joined rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct JoinedRoom {
        pub unread_notifications: UnreadNotificationsCount,
        pub timeline: Timeline,
        pub state: State,
        pub account_data: AccountData,
        pub ephemeral: Ephemeral
    }

    /// unread notifications count
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct UnreadNotificationsCount {
        pub highlight_count: u64,
        pub notification_count: u64
    }

    /// timeline
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Timeline {
        pub limited: bool,
        pub prev_batch: String,
        pub events: only::RoomEvent
    }

    /// state
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct State {
        pub events: only::StateEvent
    }

    /// account data
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AccountData {
        pub events: only::Event
    }

    /// ephemeral
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Ephemeral {
        pub events: only::Event
    }

    /// invited room updates
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InvitedRoom {
        pub invite_state: InviteState
    }

    /// invite state
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InviteState {
        pub events: only::StateEvent
    }

    /// presence
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Presence {
        pub events: only::Event
    }

    /// This API endpoint's reponse.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub next_batch: String,
        pub rooms: Rooms,
        pub presence: Presence
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = QueryParams;
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(_params: Self::PathParams) -> String {
            "/_matrix/client/r0/sync".to_string()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/sync".to_string()
        }
    }
}
