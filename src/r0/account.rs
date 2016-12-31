//! Endpoints for account registration and management.

/// [POST /_matrix/client/r0/register](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-register)
pub mod register {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bind_email: Option<bool>,
        pub password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<String>,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's query string parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<RegistrationKind>,
    }

    /// The kind of account being registered.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize)]
    pub enum RegistrationKind {
        #[serde(rename="guest")]
        Guest,
        #[serde(rename="user")]
        User,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub access_token: String,
        pub home_server: String,
        pub user_id: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = ();
        type QueryParams = QueryParams;
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/register".to_string()
        }
    }
}

/// [POST /_matrix/client/r0/account/password/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password-email-requesttoken)
pub mod request_password_change_token {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub client_secret: String,
        pub email: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,
        pub send_attempt: u64,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
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
            "/_matrix/client/r0/account/password/email/requestToken".to_string()
        }
    }
}

/// [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-deactivate)
pub mod deactivate {
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
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/account/deactivate".to_string()
        }
    }
}

/// [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password)
pub mod change_password {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub new_password: String,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
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
            "/_matrix/client/r0/account/password".to_string()
        }
    }
}

/// [POST /_matrix/client/r0/register/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-register-email-requesttoken)
pub mod request_register_token {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub client_secret: String,
        pub email: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,
        pub send_attempt: u64,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
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
            "/_matrix/client/r0/register/email/requestToken".to_string()
        }
    }
}
