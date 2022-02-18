//! `GET /_matrix/client/*/account/whoami`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3accountwhoami

    use ruma_api::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata: {
            description: "Get information about the owner of a given access token.",
            method: GET,
            name: "whoami",
            r0_path: "/_matrix/client/r0/account/whoami",
            stable_path: "/_matrix/client/v3/account/whoami",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The id of the user that owns the access token.
            pub user_id: Box<UserId>,
        }

        error: crate::Error
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user ID.
        pub fn new(user_id: Box<UserId>) -> Self {
            Self { user_id }
        }
    }
}
