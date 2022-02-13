//! `GET /.well-known/matrix/server` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/server-server-api/#getwell-knownmatrixserver

use ruma_api::ruma_api;
use ruma_identifiers::ServerName;

ruma_api! {
    metadata: {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "discover_homeserver",
        stable_path: "/.well-known/matrix/server",
        rate_limited: false,
        authentication: None,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The server name to delegate server-server communications to, with optional port.
        #[serde(rename = "m.server")]
        pub server: Box<ServerName>,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given homeserver.
    pub fn new(server: Box<ServerName>) -> Self {
        Self { server }
    }
}
