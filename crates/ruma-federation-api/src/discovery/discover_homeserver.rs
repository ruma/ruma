//! `GET /.well-known/matrix/server` ([spec])
//!
//! Get discovery information about the domain.
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#getwell-knownmatrixserver

use ruma_common::{
    api::{request, response, Metadata},
    metadata, OwnedServerName,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        1.0 => "/.well-known/matrix/server",
    }
};

/// Request type for the `discover_homeserver` endpoint.
#[request]
#[derive(Default)]
pub struct Request {}

/// Response type for the `discover_homeserver` endpoint.
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
