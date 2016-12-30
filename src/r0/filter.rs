//! Endpoints for event filters.

use ruma_identifiers::{RoomId, UserId};

/// Format to use for returned events
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum EventFormat {
    /// Client format, as described in the Client API
    #[serde(rename="client")]
    Client,
    /// Raw events from federation
    #[serde(rename="federation")]
    Federation
}

/// Filters to be applied to room events
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomEventFilter {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_types: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_rooms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub rooms: Vec<RoomId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_senders: Vec<UserId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub senders: Vec<UserId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub types: Vec<String>
}

/// Filters to be applied to room data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_leave: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<RoomEventFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<RoomEventFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<RoomEventFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RoomEventFilter>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_rooms: Vec<RoomId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub room: Vec<RoomId>
}

/// Filter for not-room data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filter {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub senders: Vec<UserId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub not_senders: Vec<UserId>
}

/// A filter definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterDefinition {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub event_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_format: Option<EventFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<RoomFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Filter>
}

/// [POST /_matrix/client/r0/user/{userId}/filter](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-user-userid-filter)
pub mod create_filter {
    use ruma_identifiers::UserId;
    use super::FilterDefinition;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API Response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub filter_id: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = FilterDefinition;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/filter",
                params.user_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/user/:user_id/filter".to_string()
        }
    }
}

/// [GET /_matrix/client/r0/user/{userId}/filter/{filterId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-user-userid-filter-filterid)
pub mod get_filter {
    use ruma_identifiers::UserId;
    use super::FilterDefinition;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub filter_id: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = FilterDefinition;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/filter/{}",
                params.user_id,
                params.filter_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/user/:user_id/filter/:filter_id".to_string()
        }
    }
}
