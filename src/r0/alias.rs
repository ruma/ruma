//! Endpoints for room aliases.

use ruma_identifiers::RoomAliasId;

/// PUT /_matrix/client/r0/directory/room/:room_alias
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-directory-room-roomalias)
pub mod create_alias {
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
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#delete-matrix-client-r0-directory-room-roomalias)
pub mod delete_alias {
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
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-directory-room-roomalias)
pub mod get_alias {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PathParams {
    pub room_alias: RoomAliasId,
}
