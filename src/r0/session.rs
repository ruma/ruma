//! Endpoints for user session management.

/// POST /_matrix/client/r0/login
pub mod login {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/login";

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub home_server: String,
        pub refresh_token: Option<String>,
        pub user_id: String,
    }
}

/// POST /_matrix/client/r0/logout
pub mod logout {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/logout";
}

///  POST /_matrix/client/r0/tokenrefresh
pub mod refresh_access_token {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/tokenrefresh";

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Request {
        pub refresh_token: String,
    }

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub refresh_token: Option<String>,
    }
}
