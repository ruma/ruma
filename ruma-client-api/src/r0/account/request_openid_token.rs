//! [POST /_matrix/client/r0/user/{userId}/openid/request_token](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-user-userid-openid-request-token)

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_common::authentication::TokenType;
use ruma_identifiers::{ServerNameBox, UserId};

ruma_api! {
    metadata: {
        description: "Request an OpenID 1.0 token to verify identity with a third party.",
        name: "request_openid_token",
        method: POST,
        path: "/_matrix/client/r0/user/:user_id/openid/request_token",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// User ID of authenticated user.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    response: {
        /// Access token for verifying user's identity.
        pub access_token: String,

        /// Access token type.
        pub token_type: TokenType,

        /// Homeserver domain for verification of user's identity.
        pub matrix_server_name: ServerNameBox,

        /// Seconds until token expiration.
        #[serde(with = "ruma_serde::duration::secs")]
        pub expires_in: Duration,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given access token, token type, server name and expiration
    /// duration.
    pub fn new(
        access_token: String,
        token_type: TokenType,
        matrix_server_name: ServerNameBox,
        expires_in: Duration,
    ) -> Self {
        Self { access_token, token_type, matrix_server_name, expires_in }
    }
}
