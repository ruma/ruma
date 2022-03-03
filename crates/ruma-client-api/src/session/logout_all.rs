//! `POST /_matrix/client/*/logout/all`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3logoutall

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Invalidates all access tokens for a user, so that they can no longer be used for authorization.",
            method: POST,
            name: "logout_all",
            r0_path: "/_matrix/client/r0/logout/all",
            stable_path: "/_matrix/client/v3/logout/all",
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
