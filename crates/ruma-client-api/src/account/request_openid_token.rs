//! `POST /_matrix/client/*/user/{userId}/openid/request_token`
//!
//! Request an OpenID 1.0 token to verify identity with a third party.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3useruseridopenidrequest_token

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        authentication::TokenType,
        metadata, OwnedServerName, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/openid/request_token",
            1.1 => "/_matrix/client/v3/user/{user_id}/openid/request_token",
        }
    };

    /// Request type for the `request_openid_token` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// User ID of authenticated user.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `request_openid_token` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
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

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
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
