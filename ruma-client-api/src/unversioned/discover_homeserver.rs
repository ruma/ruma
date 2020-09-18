//! [GET /.well-known/matrix/client](https://matrix.org/docs/spec/client_server/r0.6.0#get-well-known-matrix-client)

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "discover_homeserver",
        path: "/.well-known/matrix/client",
        rate_limited: false,
        authentication: None,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// Information about the homeserver to connect to.
        #[serde(rename = "m.homeserver")]
        pub homeserver: HomeserverInfo,

        /// Information about the identity server to connect to.
        #[serde(rename = "m.identity_server")]
        pub identity_server: Option<IdentityServerInfo>,
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
    /// Creates a `Response` with the given `HomeserverInfo`.
    pub fn new(homeserver: HomeserverInfo) -> Self {
        Self { homeserver, identity_server: None }
    }
}

/// Information about a discovered homeserver.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct HomeserverInfo {
    /// The base URL for the homeserver for client-server connections.
    pub base_url: String,
}

impl HomeserverInfo {
    /// Creates a `HomeserverInfo` with the given `base_url`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

/// Information about a discovered identity server.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct IdentityServerInfo {
    /// The base URL for the identity server for client-server connections.
    pub base_url: String,
}

impl IdentityServerInfo {
    /// Creates an `IdentityServerInfo` with the given `base_url`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}
