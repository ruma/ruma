//! `PUT /_matrix/client/*/rendezvous/{id}`
//!
//! Update a rendezvous session.

pub mod unstable {
    //! `msc4388` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4388

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: NoAuthentication,
        history: {
            unstable("io.element.msc4388") => "/_matrix/client/unstable/io.element.msc4388/rendezvous/{id}",
        }
    }

    /// Request type for the `PUT` `rendezvous` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the rendezvous session to update.
        #[ruma_api(path)]
        pub id: String,

        /// The expected sequence token for the session. If it doesn't match the server state then
        /// an error is returned.
        pub sequence_token: String,

        /// Data up to maximum size allowed by the server.
        pub data: String,
    }

    impl Request {
        /// Creates a new `Request` with the given id, sequence token and data.
        pub fn new(id: String, sequence_token: String, data: String) -> Self {
            Self { id, sequence_token, data }
        }
    }

    /// Response type for the `PUT` `rendezvous` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The new sequence token for the session.
        pub sequence_token: String,
    }

    impl Response {
        /// Creates a new `Response` with the given sequence token.
        pub fn new(sequence_token: String) -> Self {
            Self { sequence_token }
        }
    }
}
