//! `GET /_matrix/client/versions` ([spec])
//!
//! Get the versions of the client-server API supported by this homeserver.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientversions

use std::collections::{BTreeMap, BTreeSet};

use ruma_common::{
    api::{request, response, MatrixVersion, Metadata},
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ruma_common::api::MatrixVersion;

    use super::Response;

    #[test]
    fn known_versions() {
        let none = Response::new(vec![]);
        assert_eq!(none.known_versions(), BTreeSet::new());

        let single_known = Response::new(vec!["r0.6.0".to_owned()]);
        assert_eq!(single_known.known_versions(), BTreeSet::from([MatrixVersion::V1_0]));

        let single_unknown = Response::new(vec!["v0.0".to_owned()]);
        assert_eq!(single_unknown.known_versions(), BTreeSet::new());
    }

    #[test]
    fn known_versions_order() {
        let sorted = Response::new(vec![
            "r0.0.1".to_owned(),
            "r0.5.0".to_owned(),
            "r0.6.0".to_owned(),
            "r0.6.1".to_owned(),
            "v1.1".to_owned(),
            "v1.2".to_owned(),
        ]);
        assert_eq!(
            sorted.known_versions(),
            BTreeSet::from([MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2])
        );

        let sorted_reverse = Response::new(vec![
            "v1.2".to_owned(),
            "v1.1".to_owned(),
            "r0.6.1".to_owned(),
            "r0.6.0".to_owned(),
            "r0.5.0".to_owned(),
            "r0.0.1".to_owned(),
        ]);
        assert_eq!(
            sorted_reverse.known_versions(),
            BTreeSet::from([MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2])
        );

        let random_order = Response::new(vec![
            "v1.1".to_owned(),
            "r0.6.1".to_owned(),
            "r0.5.0".to_owned(),
            "r0.6.0".to_owned(),
            "r0.0.1".to_owned(),
            "v1.2".to_owned(),
        ]);
        assert_eq!(
            random_order.known_versions(),
            BTreeSet::from([MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2])
        );
    }
}
