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
    pub not_types: Option<Vec<String>>,
    pub not_rooms: Option<Vec<String>>,
    pub limit: Option<u64>,
    pub rooms: Option<Vec<RoomId>>,
    pub not_senders: Option<Vec<UserId>>,
    pub senders: Option<Vec<UserId>>,
    pub types: Option<Vec<String>>
}

/// Filters to be applied to room data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomFilter {
    pub include_leave: Option<bool>,
    pub account_data: Option<RoomEventFilter>,
    pub timeline: Option<RoomEventFilter>,
    pub ephemeral: Option<RoomEventFilter>,
    pub state: Option<RoomEventFilter>,
    pub not_rooms: Option<Vec<RoomId>>,
    pub room: Option<Vec<RoomId>>
}

/// Filter for not-room data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filter {
    pub not_types: Option<Vec<String>>,
    pub limit: Option<u64>,
    pub senders: Option<Vec<UserId>>,
    pub types: Option<Vec<String>>,
    pub not_senders: Option<Vec<UserId>>
}

/// A filter definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterDefinition {
    pub event_fields: Option<Vec<String>>,
    pub event_format: Option<EventFormat>,
    pub account_data: Option<Filter>,
    pub room: Option<RoomFilter>,
    pub presence: Option<Filter>
}

/// POST /_matrix/client/r0/user/{userId}/filter
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-user-userid-filter)
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

/// GET /_matrix/client/r0/user/{userId}/filter/{filterId}
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-user-userid-filter-filterid)
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
