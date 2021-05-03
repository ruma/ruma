//! [GET /_matrix/client/r0/voip/turnServer](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-voip-turnserver)

use std::time::Duration;

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get credentials for the client to use when initiating VoIP calls.",
        method: GET,
        name: "turn_server_info",
        path: "/_matrix/client/r0/voip/turnServer",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The username to use.
        pub username: String,

        /// The password to use.
        pub password: String,

        /// A list of TURN URIs.
        pub uris: Vec<String>,

        /// The time-to-live in seconds.
        #[serde(with = "ruma_serde::duration::secs")]
        pub ttl: Duration,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given username, password, TURN URIs and time-to-live.
    pub fn new(username: String, password: String, uris: Vec<String>, ttl: Duration) -> Self {
        Self { username, password, uris, ttl }
    }
}
