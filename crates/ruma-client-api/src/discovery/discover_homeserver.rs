//! `GET /.well-known/matrix/client` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.18/client-server-api/#getwell-knownmatrixclient
//!
//! Get discovery information about the domain.

use ruma_common::{
    api::{auth_scheme::NoAccessToken, request, response},
    metadata,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc4143")]
use crate::rtc::RtcTransport;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAccessToken,
    path: "/.well-known/matrix/client",
}

/// Request type for the `client_well_known` endpoint.
#[request]
#[derive(Default)]
pub struct Request {}

/// Response type for the `client_well_known` endpoint.
#[response]
pub struct Response {
    /// Information about the homeserver to connect to.
    #[serde(rename = "m.homeserver")]
    pub homeserver: HomeserverInfo,

    /// Information about the identity server to connect to.
    #[serde(rename = "m.identity_server", skip_serializing_if = "Option::is_none")]
    pub identity_server: Option<IdentityServerInfo>,

    /// Information about the tile server to use to display location data.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(
        rename = "org.matrix.msc3488.tile_server",
        alias = "m.tile_server",
        skip_serializing_if = "Option::is_none"
    )]
    pub tile_server: Option<TileServerInfo>,

    /// A list of the available MatrixRTC foci, ordered by priority.
    #[cfg(feature = "unstable-msc4143")]
    #[serde(
        rename = "org.matrix.msc4143.rtc_foci",
        alias = "m.rtc_foci",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub rtc_foci: Vec<RtcTransport>,
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
            #[cfg(feature = "unstable-msc4143")]
            rtc_foci: Default::default(),
        }
    }
}

/// Information about a discovered homeserver.
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
