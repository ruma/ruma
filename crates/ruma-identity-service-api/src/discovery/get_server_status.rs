//! `GET /_matrix/identity/*`
//!
//! Endpoint to check the status of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityv2

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Checks that an identity server is available at this API endpoint.",
            method: GET,
            name: "status",
            stable_path: "/_matrix/identity/v2",
            authentication: None,
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
