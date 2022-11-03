//! `GET /_matrix/client/*/pushers`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3pushers

    use ruma_common::api::ruma_api;

    use crate::push::Pusher;

    ruma_api! {
        metadata: {
            description: "Gets all currently active pushers for the authenticated user.",
            method: GET,
            name: "get_pushers",
            r0_path: "/_matrix/client/r0/pushers",
            stable_path: "/_matrix/client/v3/pushers",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// An array containing the current pushers for the user.
            pub pushers: Vec<Pusher>,
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
        /// Creates a new `Response` with the given pushers.
        pub fn new(pushers: Vec<Pusher>) -> Self {
            Self { pushers }
        }
    }
}
