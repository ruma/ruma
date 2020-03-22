//! [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.6.0.html#get-matrix-client-versions)

use std::collections::HashMap;

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Get the versions of the client-server API supported by this homeserver.",
        method: GET,
        name: "api_versions",
        path: "/_matrix/client/versions",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        /// A list of Matrix client API protocol versions supported by the homeserver.
        pub versions: Vec<String>,
        /// Experimental features supported by the server.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        pub unstable_features: HashMap<String, bool>
    }

    error: crate::Error
}
