//! Endpoints for user profiles.

/// [GET /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-displayname)
pub mod get_display_name {
    use ruma_identifiers::UserId;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

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
                "/_matrix/client/r0/profile/{}/displayname",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/profile/:user_id/displayname"
        }

        fn name() -> &'static str {
            "get_display_name"
        }

        fn description() -> &'static str {
            "Get the display name of a user."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}


/// [PUT /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-displayname)
pub mod set_display_name {
    use ruma_identifiers::UserId;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/profile/{}/displayname",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/profile/:user_id/displayname"
        }

        fn name() -> &'static str {
            "set_display_name"
        }

        fn description() -> &'static str {
            "Set the display name of the user."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [GET /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-avatar-url)
pub mod get_avatar_url {
    use ruma_identifiers::UserId;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

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
                "/_matrix/client/r0/profile/{}/avatar_url",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/profile/:user_id/avatar_url"
        }

        fn name() -> &'static str {
            "get_avatar_url"
        }

        fn description() -> &'static str {
            "Get the avatar URL of a user."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [PUT /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-avatar-url)
pub mod set_avatar_url {
    use ruma_identifiers::UserId;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/profile/{}/avatar_url",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/profile/:user_id/avatar_url"
        }

        fn name() -> &'static str {
            "set_avatar_url"
        }

        fn description() -> &'static str {
            "Set the avatar URL of the user."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [GET /_matrix/client/r0/profile/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid)
pub mod get_profile {
    use ruma_identifiers::UserId;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

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
                "/_matrix/client/r0/profile/{}",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/profile/:user_id"
        }

        fn name() -> &'static str {
            "get_profile"
        }

        fn description() -> &'static str {
            "Get all profile information of an user."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
