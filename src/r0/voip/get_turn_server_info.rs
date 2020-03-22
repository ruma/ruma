//! [GET /_matrix/client/r0/voip/turnServer](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-voip-turnserver)

use std::time::Duration;

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Get credentials for the client to use when initiating VoIP calls.",
        method: GET,
        name: "turn_server_info",
        path: "_matrix/client/r0/voip/turnServer",
        rate_limited: true,
        requires_authentication: true,
    }

    request {}

    response {
        /// The password to use.
        pub password: String,
        /// The time-to-live in seconds.
        #[serde(with = "crate::serde::duration::secs")]
        pub ttl: Duration,
        /// A list of TURN URIs.
        pub uris: Vec<String>,
        /// The username to use.
        pub username: String,
    }

    error: crate::Error
}
