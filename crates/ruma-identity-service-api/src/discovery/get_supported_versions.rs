//! `GET /_matrix/identity/versions` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityversions
//!
//! Note: This endpoint was only implemented in/after 1.1, so a 404 could indicate the server only
//! supports 1.0 endpoints. Please use [`server_status`](super::get_server_status) to
//! double-check.
//!
//! Note: This endpoint does not contain an unstable variant for 1.0.

use std::collections::BTreeMap;

use ruma_common::api::{ruma_api, MatrixVersion};

ruma_api! {
    metadata: {
        description: "Get the versions of the identity service API supported by this endpoint.",
        method: GET,
        name: "versions",
        stable_path: "/_matrix/identity/versions",
        rate_limited: false,
        authentication: None,
        added: 1.1,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of Matrix client API protocol versions supported by the endpoint.
        pub versions: Vec<String>,
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
        Self { versions }
    }

    /// Extracts known Matrix versions from this response.
    ///
    /// Matrix versions that Ruma cannot parse, or does not know about, are discarded.
    ///
    /// The versions returned will be sorted from oldest to latest. Use [`.find()`][Iterator::find]
    /// or [`.rfind()`][DoubleEndedIterator::rfind] to look for a minimum or maximum version to use
    /// given some constraint.
    pub fn known_versions(&self) -> impl Iterator<Item = MatrixVersion> + DoubleEndedIterator {
        self.versions
            .iter()
            // Parse, discard unknown versions
            .flat_map(|s| s.parse::<MatrixVersion>())
            // Map to key-value pairs where the key is the major-minor representation
            // (which can be used as a BTreeMap unlike MatrixVersion itself)
            .map(|v| (v.into_parts(), v))
            // Collect to BTreeMap
            .collect::<BTreeMap<_, _>>()
            // Return an iterator over just the values (`MatrixVersion`s)
            .into_values()
    }
}
