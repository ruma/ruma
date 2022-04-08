//! `POST /_matrix/client/*/user/{userId}/openid/request_token`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3useruseridopenidrequest_token

    use std::time::Duration;

    use ruma_common::{api::ruma_api, authentication::TokenType, OwnedServerName, UserId};

    ruma_api! {
        metadata: {
            description: "Request an OpenID 1.0 token to verify identity with a third party.",
            name: "request_openid_token",
            method: POST,
            r0_path: "/_matrix/client/r0/user/:user_id/openid/request_token",
            stable_path: "/_matrix/client/v3/user/:user_id/openid/request_token",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
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
            pub matrix_server_name: OwnedServerName,

            /// Seconds until token expiration.
            #[serde(with = "ruma_common::serde::duration::secs")]
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
        /// Creates a new `Response` with the given access token, token type, server name and
        /// expiration duration.
        pub fn new(
            access_token: String,
            token_type: TokenType,
            matrix_server_name: OwnedServerName,
            expires_in: Duration,
        ) -> Self {
            Self { access_token, token_type, matrix_server_name, expires_in }
        }
    }
}
