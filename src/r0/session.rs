//! Endpoints for user session management.

/// POST /_matrix/client/r0/login
pub mod login {
    pub const HTTP_METHOD: &'static str = "POST";
    pub const PATH: &'static str = "/_matrix/client/r0/login";

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct LoginResponse {
        pub access_token: String,
        pub home_server: String,
        pub user_id: String,
    }
}

/// POST /_matrix/client/r0/logout
pub mod logout {
    pub const HTTP_METHOD: &'static str = "POST";
    pub const PATH: &'static str = "/_matrix/client/r0/logout";
}
