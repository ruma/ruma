//! `GET /_matrix/client/*/voip/turnServer`
//!
//! Get credentials for the client to use when initiating VoIP calls.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3voipturnserver

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/voip/turnServer",
            1.1 => "/_matrix/client/v3/voip/turnServer",
        }
    };

    /// Request type for the `turn_server_info` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `turn_server_info` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The username to use.
        pub username: String,

        /// The password to use.
        pub password: String,

        /// A list of TURN URIs.
        pub uris: Vec<String>,

        /// The time-to-live in seconds.
        #[serde(with = "ruma_common::serde::duration::secs")]
        pub ttl: Duration,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given username, password, TURN URIs and time-to-live.
        pub fn new(username: String, password: String, uris: Vec<String>, ttl: Duration) -> Self {
            Self { username, password, uris, ttl }
        }
    }
}
