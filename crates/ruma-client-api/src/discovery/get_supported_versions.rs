//! `GET /_matrix/client/versions` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientversions

use std::collections::BTreeMap;

use ruma_common::api::{ruma_api, MatrixVersion};

ruma_api! {
    metadata: {
        description: "Get the versions of the client-server API supported by this homeserver.",
        method: GET,
        name: "api_versions",
        stable_path: "/_matrix/client/versions",
        rate_limited: false,
        authentication: None,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of Matrix client API protocol versions supported by the homeserver.
        pub versions: Vec<String>,

        /// Experimental features supported by the server.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub unstable_features: BTreeMap<String, bool>,
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

#[cfg(test)]
mod tests {
    use ruma_common::api::MatrixVersion;

    use super::Response;

    #[test]
    fn known_versions() {
        let none = Response::new(vec![]);
        assert_eq!(none.known_versions().next(), None);

        let single_known = Response::new(vec!["r0.6.0".to_owned()]);
        assert_eq!(single_known.known_versions().collect::<Vec<_>>(), vec![MatrixVersion::V1_0]);

        let single_unknown = Response::new(vec!["v0.0".to_owned()]);
        assert_eq!(single_unknown.known_versions().next(), None);
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
            sorted.known_versions().collect::<Vec<_>>(),
            vec![MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2],
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
            sorted_reverse.known_versions().collect::<Vec<_>>(),
            vec![MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2],
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
            random_order.known_versions().collect::<Vec<_>>(),
            vec![MatrixVersion::V1_0, MatrixVersion::V1_1, MatrixVersion::V1_2],
        );
    }
}
