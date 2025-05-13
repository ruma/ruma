//! `GET /_matrix/client/versions` ([spec])
//!
//! Get the versions of the client-server API supported by this homeserver.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientversions

use std::collections::BTreeMap;

use ruma_common::{
    api::{request, response, Metadata, SupportedVersions},
    metadata,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: AccessTokenOptional,
    history: {
        1.0 => "/_matrix/client/versions",
    }
};

/// Request type for the `api_versions` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {}

/// Response type for the `api_versions` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// A list of Matrix client API protocol versions supported by the homeserver.
    pub versions: Vec<String>,

    /// Experimental features supported by the server.
    ///
    /// Servers can enable some unstable features only for some users, so this
    /// list might differ when an access token is provided.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unstable_features: BTreeMap<String, bool>,
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
        Self { versions, unstable_features: BTreeMap::new() }
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
