//! `GET /.well-known/matrix/client` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/client-server-api/#getwell-knownmatrixclient

use ruma_common::api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "client_well_known",
        stable_path: "/.well-known/matrix/client",
        rate_limited: false,
        authentication: None,
        added: 1.0,
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

        /// Information about the tile server to use to display location data.
        #[cfg(feature = "unstable-msc3488")]
        #[serde(rename = "org.matrix.msc3488.tile_server", alias = "m.tile_server")]
        pub tile_server: Option<TileServerInfo>,
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
    /// Creates a new `Response` with the given `HomeserverInfo`.
    pub fn new(homeserver: HomeserverInfo) -> Self {
        Self {
            homeserver,
            identity_server: None,
            #[cfg(feature = "unstable-msc3488")]
            tile_server: None,
        }
    }
}

/// Information about a discovered homeserver.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct HomeserverInfo {
    /// The base URL for the homeserver for client-server connections.
    pub base_url: String,
}

impl HomeserverInfo {
    /// Creates a new `HomeserverInfo` with the given `base_url`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

/// Information about a discovered identity server.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
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

/// Information about a discovered map tile server.
#[cfg(feature = "unstable-msc3488")]
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TileServerInfo {
    /// The URL of a map tile server's `style.json` file.
    ///
    /// See the [Mapbox Style Specification](https://docs.mapbox.com/mapbox-gl-js/style-spec/) for more details.
    pub map_style_url: String,
}

#[cfg(feature = "unstable-msc3488")]
impl TileServerInfo {
    /// Creates a `TileServerInfo` with the given map style URL.
    pub fn new(map_style_url: String) -> Self {
        Self { map_style_url }
    }
}
