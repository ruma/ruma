//! Endpoints for tagging rooms.

/// [PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-user-userid-rooms-roomid-tags-tag)
pub mod create_tag {
    use ruma_identifiers::{UserId, RoomId};
    use ruma_events::tag::TagInfo;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub room_id: RoomId,
        pub tag: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = TagInfo;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/rooms/{}/tags/{}",
                params.user_id,
                params.room_id,
                params.tag
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag"
        }

        fn name() -> &'static str {
            "create_tag"
        }

        fn description() -> &'static str {
            "Add a new tag to a room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-user-userid-rooms-roomid-tags)
pub mod get_tags {
    use ruma_identifiers::{UserId, RoomId};
    use ruma_events::tag::TagEventContent;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub room_id: RoomId
    }

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub tags: TagEventContent,
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
                "/_matrix/client/r0/user/{}/rooms/{}/tags",
                params.user_id,
                params.room_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags"
        }

        fn name() -> &'static str {
            "get_tags"
        }

        fn description() -> &'static str {
            "Get the tags associated with a room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [DELETE /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.2.0.html#delete-matrix-client-r0-user-userid-rooms-roomid-tags-tag)
pub mod delete_tag {
    use ruma_identifiers::{UserId, RoomId};

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId,
        pub room_id: RoomId,
        pub tag: String
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Delete
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/user/{}/rooms/{}/tags/{}",
                params.user_id,
                params.room_id,
                params.tag
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag"
        }

        fn name() -> &'static str {
            "delete_tag"
        }

        fn description() -> &'static str {
            "Remove a tag from a room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
