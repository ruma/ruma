//! Endpoints for user session management.

/// POST /_matrix/client/r0/login
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-login)
pub mod login {
    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub home_server: String,
        pub refresh_token: Option<String>,
        pub user_id: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/login".to_string()
        }
    }
}

/// POST /_matrix/client/r0/logout
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-logout)
pub mod logout {
    /// Details about this API endpoint.
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/logout".to_string()
        }
    }
}

///  POST /_matrix/client/r0/tokenrefresh
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-tokenrefresh)
pub mod refresh_access_token {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub refresh_token: String,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub refresh_token: Option<String>,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/tokenrefresh".to_string()
        }
    }
}
