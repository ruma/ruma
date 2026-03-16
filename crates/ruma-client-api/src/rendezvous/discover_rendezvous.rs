//! `GET /_matrix/client/*/rendezvous`
//!
//! Discover if the rendezvous API is available.

pub mod unstable {
    //! `unstable/io.element.msc4388` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4388

    use ruma_common::{
        api::{auth_scheme::AccessTokenOptional, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessTokenOptional,
        history: {
            unstable("io.element.msc4388") => "/_matrix/client/unstable/io.element.msc4388/rendezvous",
        }
    }

    /// Request type for the `GET` `rendezvous` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {}

    impl Request {
        /// Creates a new `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[response(error = crate::Error)]
    /// Response type for the `GET` `rendezvous` endpoint.
    pub struct Response {}

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
