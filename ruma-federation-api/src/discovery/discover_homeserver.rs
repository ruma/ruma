//! [GET /.well-known/matrix/server](https://matrix.org/docs/spec/server_server/r0.1.3#get-well-known-matrix-server)

use ruma_api::ruma_api;
use ruma_identifiers::ServerNameBox;

ruma_api! {
    metadata: {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "discover_homeserver",
        path: "/.well-known/matrix/server",
        rate_limited: false,
        authentication: None,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The server name to delegate server-server communciations to, with optional port.
        #[serde(rename = "m.homeserver")]
        pub homeserver: ServerNameBox,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given homeserver.
    pub fn new(homeserver: ServerNameBox) -> Self {
        Self { homeserver }
    }
}
