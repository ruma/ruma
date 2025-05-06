//! `GET /_matrix/identity/versions` ([spec])
//!
//! Get the versions of the identity service API supported by this endpoint.
//!
//! Note: This endpoint was only implemented in/after 1.1, so a 404 could indicate the server only
//! supports 1.0 endpoints. Please use [`server_status`](super::get_server_status) to
//! double-check.
//!
//! Note: This endpoint does not contain an unstable variant for 1.0.
//!
//! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityversions

use std::collections::BTreeMap;

use ruma_common::{
    api::{request, response, Metadata, SupportedVersions},
    metadata,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        1.1 => "/_matrix/identity/versions",
    }
};

/// Request type for the `versions` endpoint.
#[request]
#[derive(Default)]
pub struct Request {}

/// Response type for the `versions` endpoint.
#[response]
pub struct Response {
    /// A list of Matrix client API protocol versions supported by the endpoint.
    pub versions: Vec<String>,
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
        Self { versions }
    }

    /// Convert this `Response` into a [`SupportedVersions`] that can be used with
    /// `OutgoingRequest::try_into_http_request()`.
    ///
    /// Matrix versions that can't be parsed to a `MatrixVersion`, and features with the boolean
    /// value set to `false` are discarded.
    pub fn as_supported_versions(&self) -> SupportedVersions {
        SupportedVersions::from_parts(&self.versions, &BTreeMap::new())
    }
}
