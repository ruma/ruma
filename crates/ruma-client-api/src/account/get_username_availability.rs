//! `GET /_matrix/client/*/register/available`
//!
//! Checks to see if a username is available, and valid, for the server.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3registeravailable

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/register/available",
            1.1 => "/_matrix/client/v3/register/available",
        }
    };

    /// Request type for the `get_username_availability` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The username to check the availability of.
        #[ruma_api(query)]
        pub username: String,
    }

    /// Response type for the `get_username_availability` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A flag to indicate that the username is available.
        /// This should always be true when the server replies with 200 OK.
        pub available: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given username.
        pub fn new(username: String) -> Self {
            Self { username }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given availability flag.
        pub fn new(available: bool) -> Self {
            Self { available }
        }
    }
}
