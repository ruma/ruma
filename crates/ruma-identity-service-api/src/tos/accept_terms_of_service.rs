//! `POST /_matrix/identity/*/terms`
//!
//! Send acceptance of the terms of service of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv2terms

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/terms",
        }
    }

    /// Request type for the `accept_terms_of_service` endpoint.
    #[request]
    pub struct Request {
        /// The URLs the user is accepting in this request.
        ///
        /// An example is `https://example.org/somewhere/terms-2.0-en.html`.
        pub user_accepts: Vec<String>,
    }

    /// Response type for the `accept_terms_of_service` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given URLs which the user accepts.
        pub fn new(user_accepts: Vec<String>) -> Self {
            Self { user_accepts }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
