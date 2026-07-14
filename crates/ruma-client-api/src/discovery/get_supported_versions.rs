//! `GET /_matrix/client/versions` ([spec])
//!
//! Get the versions of the client-server API supported by this homeserver.
//!
//! [spec]: https://spec.matrix.org/v1.19/client-server-api/#get_matrixclientversions

use std::collections::BTreeMap;

use ruma_common::{
    api::{SupportedVersions, auth_scheme::AccessTokenOptional, request, response},
    metadata,
};
#[cfg(feature = "unstable-msc4383")]
use serde::{Deserialize, Serialize};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: AccessTokenOptional,
    path: "/_matrix/client/versions",
}

/// Request type for the `api_versions` endpoint.
#[request]
#[derive(Default)]
pub struct Request {}

/// Response type for the `api_versions` endpoint.
#[response]
pub struct Response {
    /// A list of Matrix client API protocol versions supported by the homeserver.
    pub versions: Vec<String>,

    /// Experimental features supported by the server.
    ///
    /// Servers can enable some unstable features only for some users, so this
    /// list might differ when an access token is provided.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unstable_features: BTreeMap<String, bool>,

    /// Information about the homeserver implementation, with the same shape as
    /// the object returned by `GET /_matrix/federation/v1/version`.
    ///
    /// This uses the unstable prefix defined in [MSC4383].
    ///
    /// [MSC4383]: https://github.com/matrix-org/matrix-spec-proposals/pull/4383
    #[cfg(feature = "unstable-msc4383")]
    #[serde(rename = "net.zemos.msc4383.server", default, skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
}

/// Identifying information about the homeserver implementation.
///
/// This uses the unstable prefix defined in [MSC4383].
///
/// [MSC4383]: https://github.com/matrix-org/matrix-spec-proposals/pull/4383
#[cfg(feature = "unstable-msc4383")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct Server {
    /// Arbitrary name that identifies this implementation.
    pub name: String,

    /// Version of this implementation.
    ///
    /// The version format depends on the implementation.
    pub version: String,
}

#[cfg(feature = "unstable-msc4383")]
impl Server {
    /// Creates a `Server` with the given implementation `name` and `version`.
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given `versions`.
    pub fn new(versions: Vec<String>) -> Self {
        Self {
            versions,
            unstable_features: BTreeMap::new(),
            #[cfg(feature = "unstable-msc4383")]
            server: None,
        }
    }

    /// Convert this `Response` into a [`SupportedVersions`] that can be used with
    /// `OutgoingRequest::try_into_http_request()`.
    ///
    /// Matrix versions that can't be parsed to a `MatrixVersion`, and features with the boolean
    /// value set to `false` are discarded.
    pub fn as_supported_versions(&self) -> SupportedVersions {
        SupportedVersions::from_parts(&self.versions, &self.unstable_features)
    }
}
