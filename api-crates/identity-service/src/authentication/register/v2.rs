//! [POST /_matrix/identity/v2/account/register](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-account-register)

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_common::authentication::TokenType;
use ruma_identifiers::ServerName;

ruma_api! {
    metadata: {
        description: "Exchanges an OpenID token from the homeserver for an access token to access the identity server.",
        method: POST,
        name: "register_account",
        path: "/_matrix/identity/v2/account/register",
        authentication: None,
        rate_limited: false,
    }

    request: {
        /// An access token the consumer may use to verify the identity of the
        /// person who generated the token. This is given to the federation API
        /// GET /openid/userinfo to verify the user's identity.
        pub access_token: &'a str,

        /// The string `Bearer`.
        pub token_type: TokenType,

        /// The homeserver domain the consumer should use when attempting to verify the user's identity.
        pub matrix_server_name: &'a ServerName,

        /// The number of seconds before this token expires and a new one must be generated.
        #[serde(with = "ruma_serde::duration::secs")]
        pub expires_in: Duration,
    }

    response: {
        /// An opaque string representing the token to authenticate future requests to the identity server with.
        pub token: String,
    }
}

impl<'a> Request<'a> {
    /// Creates a `Request` with the given parameters.
    pub fn new(
        access_token: &'a str,
        token_type: TokenType,
        matrix_server_name: &'a ServerName,
        expires_in: Duration,
    ) -> Self {
        Self { access_token, token_type, matrix_server_name, expires_in }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new(token: String) -> Self {
        Self { token }
    }
}
