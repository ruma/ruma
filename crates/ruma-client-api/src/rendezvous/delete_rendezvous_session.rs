//! `DELETE /_matrix/client/*/rendezvous/{id}`
//!
//! Delete/close a rendezvous session.

pub mod unstable {
    //! `unstable/io.element.msc4388` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4388

    use ruma_common::{
        api::{auth_scheme::NoAccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: NoAccessToken,
        history: {
            unstable => "/_matrix/client/unstable/io.element.msc4388/rendezvous/{id}",
        }
    }

    /// Request type for the `DELETE` `rendezvous` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the rendezvous session to delete.
        #[ruma_api(path)]
        pub id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given id.
        pub fn new(id: String) -> Self {
            Self { id }
        }
    }

    /// Response type for the `DELETE` `rendezvous` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
