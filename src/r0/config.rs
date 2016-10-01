//! Endpoints for client configuration.

/// PUT /_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:type
pub mod set_room_account_data {
    use ruma_identifiers::{RoomId, UserId};

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub room_id: RoomId,
        pub event_type: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ::serde_json::Value;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/rooms/{}/account_data/{}",
                params.user_id,
                params.room_id,
                params.event_type
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:type".to_string()
        }
    }
}

/// PUT /_matrix/client/r0/user/:user_id/account_data/:type
pub mod set_global_account_data  {
    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub event_type: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ::serde_json::Value;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/account_data/{}",
                params.user_id,
                params.event_type
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/user/:user_id/account_data/:type".to_string()
        }
    }
}
