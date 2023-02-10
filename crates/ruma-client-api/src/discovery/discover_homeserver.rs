//! `GET /.well-known/matrix/client` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#getwell-knownmatrixclient
//!
//! Get discovery information about the domain.

use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};
use serde::{Deserialize, Serialize};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        1.0 => "/.well-known/matrix/client",
    }
};

/// Request type for the `client_well_known` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {}

/// Response type for the `client_well_known` endpoint.
#[response(error = crate::Error)]
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

    /// Information about the authentication server to connect to when using OpenID Connect.
    #[cfg(feature = "unstable-msc2965")]
    #[serde(
        rename = "org.matrix.msc2965.authentication",
        alias = "m.authentication",
        skip_serializing_if = "Option::is_none"
    )]
    pub authentication: Option<AuthenticationServerInfo>,

    /// Information about the homeserver's trusted proxy to use for sliding sync development.
    #[cfg(feature = "unstable-msc3575")]
    #[serde(rename = "org.matrix.msc3575.proxy", skip_serializing_if = "Option::is_none")]
    pub sliding_sync_proxy: Option<SlidingSyncProxyInfo>,
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
            #[cfg(feature = "unstable-msc2965")]
            authentication: None,
            #[cfg(feature = "unstable-msc3575")]
            sliding_sync_proxy: None,
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

/// Information about a discovered authentication server.
#[cfg(feature = "unstable-msc2965")]
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AuthenticationServerInfo {
    /// The OIDC Provider that is trusted by the homeserver.
    pub issuer: String,

    /// The URL where the user is able to access the account management
    /// capabilities of the OIDC Provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

#[cfg(feature = "unstable-msc2965")]
impl AuthenticationServerInfo {
    /// Creates an `AuthenticationServerInfo` with the given `issuer` and an optional `account`.
    pub fn new(issuer: String, account: Option<String>) -> Self {
        Self { issuer, account }
    }
}

/// Information about a discovered sliding sync proxy.
#[cfg(feature = "unstable-msc3575")]
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SlidingSyncProxyInfo {
    /// The URL of a sliding sync proxy that is trusted by the homeserver.
    pub url: String,
}

#[cfg(feature = "unstable-msc3575")]
impl SlidingSyncProxyInfo {
    /// Creates a `SlidingSyncProxyInfo` with the given proxy URL.
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
