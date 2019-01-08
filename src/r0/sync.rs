//! Endpoints for getting and synchronizing events.

/// [GET /_matrix/client/r0/rooms/{roomId}/state](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state)
pub mod get_state_events {
    use ruma_api_macros::ruma_api;
    use ruma_events::collections::only;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get state events for a room.",
            method: GET,
            name: "get_state_events",
            path: "/_matrix/client/r0/rooms/:room_id/state",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to look up the state for.
            #[ruma_api(path)]
            pub room_id: RoomId,
        }

        response {
            /// If the user is a member of the room this will be the current state of the room as a
            /// list of events. If the user has left the room then this will be the state of the
            /// room when they left as a list of events.
            #[ruma_api(body)]
            pub room_state: Vec<only::StateEvent>,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype)
pub mod get_state_events_for_empty_key {
    use ruma_api_macros::ruma_api;
    use ruma_events::EventType;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get state events of a given type associated with the empty key.",
            method: GET,
            name: "get_state_events_for_empty_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to look up the state for.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of state to look up.
            #[ruma_api(path)]
            pub event_type: EventType,
        }

        response {
            /// The content of the state event.
            #[ruma_api(body)]
            pub content: ::serde_json::Value,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)
pub mod get_state_events_for_key {
    use ruma_api_macros::ruma_api;
    use ruma_events::EventType;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get state events associated with a given key.",
            method: GET,
            name: "get_state_events_for_key",
            path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to look up the state for.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The type of state to look up.
            #[ruma_api(path)]
            pub event_type: EventType,
            /// The key of the state to look up.
            #[ruma_api(path)]
            pub state_key: String,
        }

        response {
            /// The content of the state event.
            #[ruma_api(body)]
            pub content: ::serde_json::Value,
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-members)
pub mod get_member_events {
    use ruma_api_macros::ruma_api;
    use ruma_events::room::member::MemberEvent;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get membership events for a room.",
            method: GET,
            name: "get_member_events",
            path: "/_matrix/client/r0/rooms/:room_id/members",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to get the member events for.
            #[ruma_api(path)]
            pub room_id: RoomId,
        }

        response {
            /// A list of member events.
            pub chunk: Vec<MemberEvent>
        }
    }
}

/// [GET /_matrix/client/r0/rooms/{roomId}/messages](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-messages)
pub mod get_message_events {
    use ruma_api_macros::ruma_api;
    use ruma_events::collections::only;
    use ruma_identifiers::RoomId;

    ruma_api! {
        metadata {
            description: "Get message events for a room.",
            method: GET,
            name: "get_message_events",
            path: "/_matrix/client/r0/rooms/:room_id/messages",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to get events from.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The token to start returning events from.
            ///
            /// This token can be obtained from a
            /// prev_batch token returned for each room by the sync API, or from a start or end token
            /// returned by a previous request to this endpoint.
            pub from: String,
            /// The token to stop returning events at.
            ///
            /// This token can be obtained from a prev_batch
            /// token returned for each room by the sync endpoint, or from a start or end token returned
            /// by a previous request to this endpoint.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub to: Option<String>,
            /// The direction to return events from.
            pub dir: Direction,
            /// The maximum number of events to return.
            ///
            /// Default: 10.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub limit: Option<u64>,
        }

        response {
            /// The token the pagination starts from.
            pub start: String,
            /// A list of room events.
            pub chunk: Vec<only::RoomEvent>,
            /// The token the pagination ends at.
            pub end: String,
        }
    }

    /// The direction to return events from.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum Direction {
        /// Return events backwards in time from the requested `from` token.
        #[serde(rename="b")]
        Backward,
        /// Return events forwards in time from the requested `from` token.
        #[serde(rename="f")]
        Forward,
    }
}

/// [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-sync)
pub mod sync_events {
    use std::collections::HashMap;

    use ruma_api_macros::ruma_api;
    use ruma_events::{collections::{all, only}, stripped};
    use ruma_identifiers::RoomId;

    use r0::filter::FilterDefinition;

    ruma_api! {
        metadata {
            description: "Get all new events from all rooms since the last sync or a given point of time.",
            method: GET,
            name: "sync",
            path: "/_matrix/client/r0/sync",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// A filter represented either as its full JSON definition or the ID of a saved filter.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub filter: Option<Filter>,
            /// A point in time to continue a sync from.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub since: Option<String>,
            /// Controls whether to include the full state for all rooms the user is a member of.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub full_state: Option<bool>,
            /// Controls whether the client is automatically marked as online by polling this API.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub set_presence: Option<SetPresence>,
            /// The maximum time to poll in milliseconds before returning this request.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub timeout: Option<u64>,
        }

        response {
            /// The batch token to supply in the `since` param of the next `/sync` request.
            pub next_batch: String,
            /// Updates to rooms.
            pub rooms: Rooms,
            /// Updates to the presence status of other users.
            pub presence: Presence,
        }

    }

    /// Whether to set presence or not during sync.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum SetPresence {
        /// Do not set the presence of the user calling this API.
        #[serde(rename="offline")]
        Offline,
    }

    /// A filter represented either as its full JSON definition or the ID of a saved filter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum Filter {
        /// A complete filter definition serialized to JSON.
        FilterDefinition(FilterDefinition),
        /// The ID of a filter saved on the server.
        FilterId(String),
    }

    /// Updates to rooms.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Rooms {
        /// The rooms that the user has left or been banned from.
        pub leave: HashMap<RoomId, LeftRoom>,
        /// The rooms that the user has joined.
        pub join: HashMap<RoomId, JoinedRoom>,
        /// The rooms that the user has been invited to.
        pub invite: HashMap<RoomId, InvitedRoom>,
    }

    /// Historical updates to left rooms.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LeftRoom {
        /// The timeline of messages and state changes in the room up to the point when the user
        /// left.
        pub timeline: Timeline,
        /// The state updates for the room up to the start of the timeline.
        pub state: State,
    }

    /// Updates to joined rooms.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct JoinedRoom {
        /// Counts of unread notifications for this room.
        pub unread_notifications: UnreadNotificationsCount,
        /// The timeline of messages and state changes in the room.
        pub timeline: Timeline,
        /// Updates to the state, between the time indicated by the `since` parameter, and the start
        /// of the `timeline` (or all state up to the start of the `timeline`, if `since` is not
        /// given, or `full_state` is true).
        pub state: State,
        /// The private data that this user has attached to this room.
        pub account_data: AccountData,
        /// The ephemeral events in the room that aren't recorded in the timeline or state of the
        /// room. e.g. typing.
        pub ephemeral: Ephemeral,
    }

    /// unread notifications count
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct UnreadNotificationsCount {
        /// The number of unread notifications for this room with the highlight flag set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub highlight_count: Option<u64>,
        /// The total number of unread notifications for this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub notification_count: Option<u64>,
    }

    /// Events in the room.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Timeline {
        /// True if the number of events returned was limited by the `limit` on the filter.
        pub limited: bool,
        /// A token that can be supplied to to the `from` parameter of the
        /// `/rooms/{roomId}/messages` endpoint.
        pub prev_batch: String,
        /// A list of events.
        pub events: Vec<all::RoomEvent>,
    }

    /// State events in the room.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct State {
        /// A list of state events.
        pub events: Vec<only::StateEvent>,
    }

    /// The private data that this user has attached to this room.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AccountData {
        /// A list of events.
        pub events: Vec<only::Event>,
    }

    /// Ephemeral events not recorded in the timeline or state of the room.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Ephemeral {
        /// A list of events.
        pub events: Vec<only::Event>,
    }

    /// Updates to the rooms that the user has been invited to.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InvitedRoom {
        /// The state of a room that the user has been invited to.
        pub invite_state: InviteState,
    }

    /// The state of a room that the user has been invited to.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct InviteState {
        /// A list of state events.
        pub events: Vec<stripped::StrippedState>,
    }

    /// Updates to the presence status of other users.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Presence {
        /// A list of events.
        pub events: Vec<only::Event>,
    }
}
