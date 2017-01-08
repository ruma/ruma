//! Endpoints for account contact information.

/// [POST /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-3pid)
pub mod create_contact {
    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bind: Option<bool>,
        pub three_pid_creds: ThreePidCredentials,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// The third party credentials to associate with the account.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ThreePidCredentials {
        pub client_secret: String,
        pub id_server: String,
        pub sid: String,
    }

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
            "/_matrix/client/r0/account/3pid"
        }

        fn name() -> &'static str {
            "create_contact"
        }

        fn description() -> &'static str {
            "Adds contact information to the user's account."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [GET /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-account-3pid)
pub mod get_contacts {
    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// The medium of third party identifier.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum Medium {
        #[serde(rename="email")]
        Email,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub threepids: Vec<ThirdPartyIdentifier>,
    }

    /// An identifier external to Matrix.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ThirdPartyIdentifier {
        pub address: String,
        pub medium: Medium,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/account/3pid/email/requestToken"
        }

        fn name() -> &'static str {
            "get_contacts"
        }

        fn description() -> &'static str {
            "Get a list of 3rd party contacts associated with the user's account."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [POST /_matrix/client/r0/account/3pid/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-3pid-email-requesttoken)
pub mod request_contact_verification_token {
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
            "/_matrix/client/r0/account/3pid/email/requestToken"
        }

        fn name() -> &'static str {
            "request_contact_verification_token"
        }

        fn description() -> &'static str {
            "Ask for a verification token for a given 3rd party ID."
        }

        fn requires_authentication() -> bool {
            // Not sure why this don't require auth?
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
