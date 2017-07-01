//! Endpoints for getting and synchronizing events.

/// [GET /_matrix/client/r0/rooms/{roomId}/state](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state)
pub mod get_state_events {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use ruma_events::collections::only;

    ruma_api! {
        metadata {
            description: "Get state events for a room.",
            method: Method::Get,
            name: "get_state_events",
            path: "/_matrix/client/r0/rooms/:room_id/state",
            rate_limited: false,
            requires_authentication: true,
        }
        request {
            #[ruma_api(path)]
            pub room_id: RoomId,
        }
        response {
            pub room_state: Vec<only::StateEvent>,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod get_state_events_for_empty_key {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use ruma_events::EventType;

    ruma_api! {
        metadata {
            description: "Get state events of a given type associated with the empty key.",
            method: Method::Get,
            name: "get_state_events_for_empty_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
            rate_limited: false,
            requires_authentication: true,
        }
        request {
            /// The room to query for events
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of state to look up
            #[ruma_api(path)]
            pub event_type: EventType,
        }
        response {
            pub content: ::serde_json::Value,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)
pub mod get_state_events_for_key {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get state events associated with a given key.",
            method: Method::Get,
            name: "get_state_events_for_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
            rate_limited: false,
            requires_authentication: true,
        }
        request {
            /// The room to look up the state in.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of state to look up.
            #[ruma_api(path)]
            pub event_type: String,
            /// The key of the state to look up.
            #[ruma_api(path)]
            pub state_key: String,
        }
        response {
            pub content: ::serde_json::Value,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-members)
pub mod get_member_events {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use ruma_events::room::member::MemberEvent;

    ruma_api! {
        metadata {
            description: "Get membership events for a room.",
            method: Method::Get,
            name: "get_member_events",
            path: "/_matrix/client/r0/rooms/:room_id/members",
            rate_limited: false,
            requires_authentication: false,
            // TODO: not marked as requiring auth in the spec, but
            // will return a 403 error is user is not a member of the
            // room anyway...
        }
        request {
            /// The room to look up the state in.
            #[ruma_api(path)]
            pub room_id: RoomId,
        }
        response {
            pub chunk: Vec<MemberEvent>
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/messages](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-messages)
pub mod get_message_events {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use ruma_events::collections::only;

    ruma_api! {
        metadata {
            description: "Get message events for a room.",
            method: Method::Get,
            name: "get_message_events",
            path: "/_matrix/client/r0/rooms/:room_id/messages",
            rate_limited: false,
            requires_authentication: true,
        }
        request {
            // NOTE: The non-macro version of this call included two path params, where the spec only
            // has one, room_id. I've followed the spec here. -- rschulman 6/30/2017
            /// The room to look up the state in.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// Required. The token to start returning events from. This token can be obtained from a 
            /// prev_batch token returned for each room by the sync API, or from a start or end token 
            /// returned by a previous request to this endpoint.
            pub from: String,
            /// The token to stop returning events at. This token can be obtained from a prev_batch 
            /// token returned for each room by the sync endpoint, or from a start or end token returned 
            /// by a previous request to this endpoint.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub to: Option<String>,
            /// Required. The direction to return events from. One of: ["b", "f"]
            pub dir: Direction,
            /// The maximum number of events to return. Default: 10.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub limit: Option<u64>,
        }
        response {
            pub start: String,
            pub chunk: Vec<only::RoomEvent>,
            pub end: String,
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum Direction {
        #[serde(rename="b")]
        Backward,
        #[serde(rename="f")]
        Forward,
    }
}

/// [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-sync)
pub mod sync_events {
    use ruma_api_macros::ruma_api;
    use ruma_events::collections::only;
    use ruma_identifiers::RoomId;

    use std::collections::HashMap;

    use r0::filter::FilterDefinition;

    ruma_api! {
        metadata {
            description: "Get all new events from all rooms since the last sync or a given point of time.",
            method: Method::Get,
            name: "sync",
            path: "/_matrix/client/r0/sync",
            rate_limited: false,
            requires_authentication: true,
        }
        request {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub filter: Option<Filter>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub since: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub full_state: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub set_presence: Option<SetPresence>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub timeout: Option<u64>,
        }
        response {
            pub next_batch: String,
            pub rooms: Rooms,
            pub presence: Presence,
        }

    }

    /// Whether to set presence or not during sync.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum SetPresence {
        #[serde(rename="offline")]
        Offline,
    }

    /// A filter represented either as its full JSON definition or the ID of a saved filter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum Filter {
        /// A complete filter definition serialized to JSON.
        FilterDefinition(FilterDefinition),
        /// The ID of a filter saved on the server.
        FilterId(String),
    }

    /// Updates to rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Rooms {
        pub leave: HashMap<RoomId, LeftRoom>,
        pub join: HashMap<RoomId, JoinedRoom>,
        pub invite: HashMap<RoomId, InvitedRoom>,
    }

    /// Historical updates to left rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LeftRoom {
        pub timeline: Timeline,
        pub state: State,
    }

    /// Updates to joined rooms
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct JoinedRoom {
        pub unread_notifications: UnreadNotificationsCount,
        pub timeline: Timeline,
        pub state: State,
        pub account_data: AccountData,
        pub ephemeral: Ephemeral,
    }

    /// unread notifications count
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct UnreadNotificationsCount {
        pub highlight_count: u64,
        pub notification_count: u64,
    }

    /// timeline
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Timeline {
        pub limited: bool,
        pub prev_batch: String,
        pub events: only::RoomEvent,
    }

    /// state
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct State {
        pub events: only::StateEvent,
    }

    /// account data
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AccountData {
        pub events: only::Event,
    }

    /// ephemeral
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Ephemeral {
        pub events: only::Event,
    }

    /// invited room updates
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InvitedRoom {
        pub invite_state: InviteState,
    }

    /// invite state
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InviteState {
        pub events: only::StateEvent,
    }

    /// presence
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Presence {
        pub events: only::Event,
    }
}
