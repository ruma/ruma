//! `GET /.well-known/matrix/server` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.4/server-server-api/#getwell-knownmatrixserver

use ruma_common::{
    api::{request, response, Metadata},
    metadata, OwnedServerName,
};

const METADATA: Metadata = metadata! {
    description: "Get discovery information about the domain.",
    method: GET,
    name: "discover_homeserver",
    rate_limited: false,
    authentication: None,
    history: {
        1.0 => "/.well-known/matrix/server",
    }
};

#[request]
#[derive(Default)]
pub struct Request {}

#[response]
pub struct Response {
    /// The server name to delegate server-server communications to, with optional port.
    #[serde(rename = "m.server")]
    pub server: OwnedServerName,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given homeserver.
    pub fn new(server: OwnedServerName) -> Self {
        Self { server }
    }
}
