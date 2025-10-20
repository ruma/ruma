//! `GET /_matrix/identity/*`
//!
//! Check the status of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityv2

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/identity/v2",
        }
    }

    /// Request type for the `status` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `status` endpoint.
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
