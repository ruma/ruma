//! [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-versions)

use std::collections::BTreeMap;

use ruma_api::ruma_api;

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
        pub unstable_features: BTreeMap<String, bool>
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a `Response` with the given `versions`.
    pub fn new(versions: Vec<String>) -> Self {
        Self { versions, unstable_features: BTreeMap::new() }
    }
}
