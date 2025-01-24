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

use std::collections::BTreeSet;

use ruma_common::{
    api::{request, response, MatrixVersion, Metadata},
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

    /// Extracts known Matrix versions from this response.
    ///
    /// Matrix versions that Ruma cannot parse, or does not know about, are discarded.
    ///
    /// The versions returned will be sorted from oldest to latest. Use [`.find()`][Iterator::find]
    /// or [`.rfind()`][DoubleEndedIterator::rfind] to look for a minimum or maximum version to use
    /// given some constraint.
    pub fn known_versions(&self) -> BTreeSet<MatrixVersion> {
        self.versions
            .iter()
            // Parse, discard unknown versions
            .flat_map(|s| s.parse::<MatrixVersion>())
            // Collect to BTreeSet
            .collect()
    }
}
