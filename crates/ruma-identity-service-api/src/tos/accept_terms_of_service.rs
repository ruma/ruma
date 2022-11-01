//! `POST /_matrix/identity/*/terms`
//!
//! Endpoint to send acceptance of the terms of service of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#post_matrixidentityv2terms

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Called by a client to indicate that the user has accepted/agreed to the included set of URLs.",
        method: POST,
        name: "accept_terms_of_service",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/terms",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The URLs the user is accepting in this request.
        ///
        /// An example is `https://example.org/somewhere/terms-2.0-en.html`.
        pub user_accepts: &'a [String],
    }

    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given URLs which the user accepts.
        pub fn new(user_accepts: &'a [String]) -> Self {
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
