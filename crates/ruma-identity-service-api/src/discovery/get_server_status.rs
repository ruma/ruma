//! `GET /_matrix/identity/*`
//!
//! Endpoint to check the status of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Checks that an identity server is available at this API endpoint.",
        method: GET,
        name: "status",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/identity/v2",
        }
    };

    #[request]
    #[derive(Default)]
    pub struct Request {}

    #[response]
    #[derive(Default)]
    pub struct Response {}

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
