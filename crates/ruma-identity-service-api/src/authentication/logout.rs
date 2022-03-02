//! `POST /_matrix/identity/*/account/logout`
//!
//! Logs out the access token, preventing it from being used to authenticate future requests to the
//! server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#post_matrixidentityv2accountlogout

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Logs out the access token, preventing it from being used to authenticate future requests to the server.",
            method: POST,
            name: "logout",
            stable_path: "/_matrix/identity/v2/account/logout",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        #[derive(Default)]
        response: {}
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
