//! Endpoints for user session management.

/// [POST /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-login)
pub mod login {
    use ruma_api_macros::ruma_api;

    ruma_api! {
    metadata {
        description: "Login to the homeserver.",
        method: ::Method::Post,
        name: "login",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        requires_authentication: false,
    }
    request {
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
    response {
        pub access_token: String,
        pub home_server: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,
        pub user_id: String,
    }

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
}

/// [POST /_matrix/client/r0/logout](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-logout)
pub mod logout {
    use ruma_api_macros::ruma_api;

    ruma_api! {
    metadata {
        description: "Log out of the homeserver.",
        method: ::Method::Post,
        name: "logout",
        path: "/_matrix/client/r0/logout",
        rate_limited: false,
        requires_authentication: true,
    }
    request {}
    response {}
}
