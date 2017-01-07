//! Endpoints for user session management.

/// [POST /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-login)
pub mod login {
    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// Possible login mediums for 3rd party ID
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum LoginMedium {
        #[serde(rename = "email")]
        Email
    }

    /// Possible kinds of login
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum LoginKind {
        #[serde(rename = "m.login.password")]
        Password
    }

    /// The body parameters for this endpoint
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        /// Password of the user
        pub password: String,
        /// Medium of 3rd party login to use
        #[serde(skip_serializing_if = "Option::is_none")]
        pub medium: Option<LoginMedium>,
        /// Type of login to do
        #[serde(rename = "type")]
        pub kind: LoginKind,
        /// Localpart or full matrix user id of the user
        pub user: String,
        /// 3rd party identifier for the user
        #[serde(skip_serializing_if = "Option::is_none")]
        pub address: Option<String>
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub home_server: String,
        #[serde(skip_serializing_if = "Option::is_none")]
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/login"
        }

        fn name() -> &'static str {
            "login"
        }

        fn description() -> &'static str {
            "Login to the homeserver."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [POST /_matrix/client/r0/logout](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-logout)
pub mod logout {
    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/logout"
        }

        fn name() -> &'static str {
            "logout"
        }

        fn description() -> &'static str {
            "Log out of the homeserver."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
