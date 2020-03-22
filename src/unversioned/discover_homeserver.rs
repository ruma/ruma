//! [GET /.well-known/matrix/client](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-well-known-matrix-client)

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};
use url::Url;

/// Information about a discovered homeserver.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, PartialOrd, Serialize)]
pub struct HomeserverInfo {
    /// The base URL for the homeserver for client-server connections.
    pub base_url: Url,
}

/// Information about a discovered identity server.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, PartialOrd, Serialize)]
pub struct IdentityServerInfo {
    /// The base URL for the identity server for client-server connections.
    pub base_url: Url,
}

ruma_api! {
    metadata {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "discover_homeserver",
        path: "/.well-known/matrix/client",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        /// Information about the homeserver to connect to.
        #[serde(rename = "m.homeserver")]
        pub homeserver: HomeserverInfo,

        /// Information about the identity server to connect to.
        #[serde(rename = "m.identity_server")]
        pub identity_server: Option<IdentityServerInfo>,
    }

    error: crate::Error
}
