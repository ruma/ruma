//! `GET /_matrix/client/*/voip/turnServer`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3voipturnserver

    use std::time::Duration;

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Get credentials for the client to use when initiating VoIP calls.",
            method: GET,
            name: "turn_server_info",
            r0_path: "/_matrix/client/r0/voip/turnServer",
            stable_path: "/_matrix/client/v3/voip/turnServer",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
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
            #[serde(with = "ruma_common::serde::duration::secs")]
            pub ttl: Duration,
        }

        error: crate::Error
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
