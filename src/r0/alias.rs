//! Endpoints for room aliases.

use ruma_identifiers::RoomAliasId;

/// PUT /_matrix/client/r0/directory/room/:room_alias
pub mod create {
    use ruma_identifiers::RoomId;

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub room_id: RoomId,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = super::PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!("/_matrix/client/r0/directory/room/{}", params.room_alias)
        }

        fn router_path() -> String {
            "/_matrix/client/r0/directory/room/:room_alias".to_string()
        }
    }
}

/// DELETE /_matrix/client/r0/directory/room/:room_alias
pub mod delete {
    /// Details about this API endpoint.
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = super::PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Delete
        }

        fn request_path(params: Self::PathParams) -> String {
            format!("/_matrix/client/r0/directory/room/{}", params.room_alias)
        }

        fn router_path() -> String {
            "/_matrix/client/r0/directory/room/:room_alias".to_string()
        }
    }
}

/// GET /_matrix/client/r0/directory/room/:room_alias
pub mod get {
    use ruma_identifiers::RoomId;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub room_id: RoomId,
        pub servers: Vec<String>,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = super::PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!("/_matrix/client/r0/directory/room/{}", params.room_alias)
        }

        fn router_path() -> String {
            "/_matrix/client/r0/directory/room/:room_alias".to_string()
        }
    }
}

/// These API endpoints' path parameters.
pub struct PathParams {
    pub room_alias: RoomAliasId,
}
