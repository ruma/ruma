//! Endpoints for account registration and management.

/// POST /_matrix/client/r0/register
pub mod register {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/register";

    /// The kind of account being registered.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize)]
    pub enum RegistrationKind {
        #[serde(rename="guest")]
        Guest,
        #[serde(rename="user")]
        User,
    }

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Request {
        pub bind_email: Option<bool>,
        pub kind: Option<RegistrationKind>,
        pub password: String,
        pub username: Option<String>,
    }

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub home_server: String,
        pub user_id: String,
    }
}

/// POST /_matrix/client/r0/account/password/email/requestToken
pub mod request_password_change_token {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/account/password/email/requestToken";
}

/// POST /_matrix/client/r0/account/deactivate
pub mod deactivate {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/account/deactivate";
}

/// POST /_matrix/client/r0/account/password
pub mod change_password {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/account/password";

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Request {
        pub new_password: String,
    }
}

/// POST /_matrix/client/r0/register/email/requestToken
pub mod request_register_token {
    /// The HTTP method.
    pub const METHOD: &'static str = "POST";

    /// The URL's path component.
    pub const PATH: &'static str = "/_matrix/client/r0/register/email/requestToken";

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Request {
        pub client_secret: String,
        pub email: String,
        pub id_server: Option<String>,
        pub send_attempt: u64,
    }
}
