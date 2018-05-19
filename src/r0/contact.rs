//! Endpoints for account contact information.

/// [POST /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-3pid)
pub mod create_contact {
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Adds contact information to the user's account.",
            method: POST,
            name: "create_contact",
            path: "/_matrix/client/r0/account/3pid",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// Whether the homeserver should also bind this third party identifier to the account's
            /// Matrix ID with the passed identity server.
            ///
            /// Default to `false` if not supplied.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub bind: Option<bool>,
            /// The third party credentials to associate with the account.
            pub three_pid_creds: ThreePidCredentials,
        }

        response {}
    }

    /// The third party credentials to associate with the account.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ThreePidCredentials {
        /// The client secret used in the session with the identity server.
        pub client_secret: String,
        /// The identity server to use.
        pub id_server: String,
        /// The session identifier given by the identity server.
        pub sid: String,
    }
}

/// [GET /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-account-3pid)
pub mod get_contacts {
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Get a list of 3rd party contacts associated with the user's account.",
            method: GET,
            name: "get_contacts",
            path: "/_matrix/client/r0/account/3pid",
            rate_limited: false,
            requires_authentication: true,
        }

        request {}

        response {
            /// A list of third party identifiers the homeserver has associated with the user's
            /// account.
            pub threepids: Vec<ThirdPartyIdentifier>,
        }
    }

    /// The medium of third party identifier.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum Medium {
        /// An email address.
        #[serde(rename="email")]
        Email,
    }

    /// An identifier external to Matrix.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ThirdPartyIdentifier {
        /// The third party identifier address.
        pub address: String,
        /// The medium of third party identifier.
        pub medium: Medium,
    }
}

/// [POST /_matrix/client/r0/account/3pid/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-3pid-email-requesttoken)
pub mod request_contact_verification_token {
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Ask for a verification token for a given 3rd party ID.",
            method: POST,
            name: "request_contact_verification_token",
            path: "/_matrix/client/r0/account/3pid/email/requestToken",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// Client-generated secret string used to protect this session.
            pub client_secret: String,
            /// The email address.
            pub email: String,
            /// The ID server to send the onward request to as a hostname with an appended colon and port number if the port is not the default.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub id_server: Option<String>,
            /// Used to distinguish protocol level retries from requests to re-send the email.
            pub send_attempt: u64,
        }

        response {}
    }
}
