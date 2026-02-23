//! `POST /_matrix/identity/*/account/register`
//!
//! Exchanges an OpenID token from the homeserver for an access token to access the identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv2accountregister

    use std::time::Duration;

    use ruma_common::{
        ServerName,
        api::{auth_scheme::NoAuthentication, request, response},
        authentication::TokenType,
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/identity/v2/account/register",
        }
    }

    /// Request type for the `register_account` endpoint.
    #[request]
    pub struct Request {
        /// An access token the consumer may use to verify the identity of the person who generated
        /// the token.
        ///
        /// This is given to the federation API `GET /openid/userinfo` to verify the user's
        /// identity.
        pub access_token: String,

        /// The string `Bearer`.
        pub token_type: TokenType,

        /// The homeserver domain the consumer should use when attempting to verify the user's
        /// identity.
        pub matrix_server_name: ServerName,

        /// The number of seconds before this token expires and a new one must be generated.
        #[serde(with = "ruma_common::serde::duration::secs")]
        pub expires_in: Duration,
    }

    /// Response type for the `register_account` endpoint.
    #[response]
    pub struct Response {
        /// An opaque string representing the token to authenticate future requests to the identity
        /// server with.
        pub token: String,
    }

    impl Request {
        /// Creates a new `Request` with the given parameters.
        pub fn new(
            access_token: String,
            token_type: TokenType,
            matrix_server_name: ServerName,
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
}
