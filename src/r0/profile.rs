//! Endpoints for user profiles.

/// GET /_matrix/client/r0/profile/{userId}/displayname
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-displayname)
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

        fn router_path() -> String {
            "/_matrix/client/r0/profile/:user_id/displayname".to_string()
        }
    }
}


/// PUT /_matrix/client/r0/profile/{userId}/displayname
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-displayname)
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

        fn router_path() -> String {
            "/_matrix/client/r0/profile/:user_id/displayname".to_string()
        }
    }
}

/// GET /_matrix/client/r0/profile/{userId}/avatar_url
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-avatar-url)
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

        fn router_path() -> String {
            "/_matrix/client/r0/profile/:user_id/avatar_url".to_string()
        }
    }
}

/// PUT /_matrix/client/r0/profile/{userId}/avatar_url
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-avatar-url)
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

        fn router_path() -> String {
            "/_matrix/client/r0/profile/:user_id/avatar_url".to_string()
        }
    }
}

/// GET /_matrix/client/r0/profile/{userId}
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid)
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

        fn router_path() -> String {
            "/_matrix/client/r0/profile/:user_id".to_string()
        }
    }
}
