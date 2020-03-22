//! [POST /_matrix/client/r0/user/{userId}/openid/request_token](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-user-userid-openid-request-token)

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Request an OpenID 1.0 token to verify identity with a third party.",
        name: "request_openid_token",
        method: POST,
        path: "/_matrix/client/r0/user/:user_id/openid/request_token",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// User ID of authenticated user.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// Access token for verifying user's identity.
        pub access_token: String,
        /// Access token type.
        pub token_type: TokenType,
        /// Homeserver domain for verification of user's identity.
        pub matrix_server_name: String,
        /// Seconds until token expiration.
        #[serde(with = "crate::serde::duration::secs")]
        pub expires_in: Duration,
    }

    error: crate::Error
}

/// Access token types.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TokenType {
    /// Bearer token type
    Bearer,
}
