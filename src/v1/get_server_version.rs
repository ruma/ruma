use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get the implementation name and version of this homeserver.",
        method: GET,
        name: "discover_homeserver",
        path: "/.well-known/matrix/server",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        /// Information about the homeserver implementation
        server: Server,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    /// Arbitrary name that identify this implementation.
    name: Option<String>,
    /// Version of this implementation. The version format depends on the implementation.
    version: Option<String>,
}
