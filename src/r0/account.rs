//! Endpoints for account registration and management.

/// [POST /_matrix/client/r0/register](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-register)
pub mod register {
    use ruma_identifiers::UserId;

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        /// If true, the server binds the email used for authentication
        /// to the Matrix ID with the ID Server.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bind_email: Option<bool>,
        /// The desired password for the account.
        ///
        /// Should only be empty for guest accounts.
        // TODO: the spec says nothing about when it is actually required.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub password: Option<String>,
        /// local part of the desired Matrix ID.
        ///
        /// If omitted, the homeserver MUST generate a Matrix ID local part.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<String>,
        /// ID of the client device.
        ///
        /// If this does not correspond to a known client device, a new device will be created.
        /// The server will auto-generate a device_id if this is not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<String>,
        /// A display name to assign to the newly-created device.
        ///
        /// Ignored if `device_id` corresponds to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<String>,
        /// Additional authentication information for the user-interactive authentication API.
        ///
        /// Note that this information is not used to define how the registered user should be
        /// authenticated, but is instead used to authenticate the register call itself.
        /// It should be left empty, or omitted, unless an earlier call returned an response
        /// with status code 401.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthenticationData>
    }

    /// Additional authentication information for the user-interactive authentication API.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AuthenticationData {
        /// The login type that the client is attempting to complete.
        #[serde(rename = "type")]
        kind: String,
        /// The value of the session key given by the homeserver.
        session: Option<String>
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's query string parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        /// Kind of account to register
        ///
        /// Defaults to `User` if ommited.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<RegistrationKind>,
    }

    /// The kind of account being registered.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize)]
    pub enum RegistrationKind {
        /// A guest account
        ///
        /// These accounts may have limited permissions and may not be supported by all servers.
        #[serde(rename="guest")]
        Guest,
        /// A regular user account
        #[serde(rename="user")]
        User,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        /// An access token for the account.
        ///
        /// This access token can then be used to authorize other requests.
        pub access_token: String,
        /// The hostname of the homeserver on which the account has been registered.
        pub home_server: String,
        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: UserId,
        /// ID of the registered device.
        ///
        /// Will be the same as the corresponding parameter in the request, if one was specified.
        pub device_id: String
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/register"
        }

        fn name() -> &'static str {
            "register"
        }

        fn description() -> &'static str {
            "Register an account on this homeserver."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [POST /_matrix/client/r0/account/password/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password-email-requesttoken)
pub mod request_password_change_token {
    // TODO: according to the spec, this does not has any params
    // probably the spec's fault, as this would not make any sense.
    // But the BodyParams here are probably wrong
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/account/password/email/requestToken"
        }

        fn name() -> &'static str {
            "request_password_change_token"
        }

        fn description() -> &'static str {
            "Request that a password change token is sent to the given email address."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-deactivate)
pub mod deactivate {
    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    // TODO: missing BodyParams

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
            "/_matrix/client/r0/account/deactivate"
        }

        fn name() -> &'static str {
            "deactivate"
        }

        fn description() -> &'static str {
            "Deactivate the current user's account."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password)
pub mod change_password {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub new_password: String,
        // TODO: missing `auth` field
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/account/password"
        }

        fn name() -> &'static str {
            "change_password"
        }

        fn description() -> &'static str {
            "Change the password of the current user's account."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/register/email/requestToken"
        }

        fn name() -> &'static str {
            "request_register_token"
        }

        fn description() -> &'static str {
            "Request a register token with a 3rd party email."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}
