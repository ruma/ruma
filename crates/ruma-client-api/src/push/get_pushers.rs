//! `GET /_matrix/client/*/pushers`
//!
//! Gets all currently active pushers for the authenticated user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3pushers

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::Pusher;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushers",
            1.1 => "/_matrix/client/v3/pushers",
        }
    };

    /// Request type for the `get_pushers` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_pushers` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// An array containing the current pushers for the user.
        pub pushers: Vec<Pusher>,
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
