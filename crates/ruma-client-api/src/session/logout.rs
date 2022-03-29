//! `POST /_matrix/client/*/logout`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3logout

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Log out of the homeserver.",
            method: POST,
            name: "logout",
            r0_path: "/_matrix/client/r0/logout",
            stable_path: "/_matrix/client/v3/logout",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        #[derive(Default)]
        response: {}

        error: crate::Error
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
