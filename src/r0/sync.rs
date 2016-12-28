//! Endpoints for getting and synchronizing events.

/// GET /_matrix/client/r0/sync
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-sync)
pub mod sync {
    use ruma_identifiers::RoomId;
    use ruma_events::collections::only;
    use std::collections::HashMap;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// Wether to set presence or not during sync
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum SetPresence {
        #[serde(rename="offline")]
        Offline
    }

    /// This API endpoint's query parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filter: Option<String>,
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
