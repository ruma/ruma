//! [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-versions)

use std::{
    collections::{BTreeMap, HashSet},
    convert::TryInto as _,
};

use ruma_api::{ruma_api, MatrixVersion};

ruma_api! {
    metadata: {
        description: "Get the versions of the client-server API supported by this homeserver.",
        method: GET,
        name: "api_versions",
        path: "/_matrix/client/versions",
        rate_limited: false,
        authentication: None,
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

    /// Extracts known ruma versions from this response.
    ///
    /// Matrix versions that ruma cannot parse, or does not know about, are discarded.
    pub fn known_versions(&self) -> Vec<MatrixVersion> {
        let mut set = HashSet::<MatrixVersion>::new();
        for s in &self.versions {
            if let Ok(ver) = s.as_str().try_into() {
                set.insert(ver);
            }
        }
        set.into_iter().collect()
    }
}
