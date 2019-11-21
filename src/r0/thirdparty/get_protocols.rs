//! [GET /_matrix/client/r0/thirdparty/protocols](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-protocols)

use std::collections::HashMap;

use ruma_api::ruma_api;

use super::Protocol;

ruma_api! {
    metadata {
        description: "Fetches the overall metadata about protocols supported by the homeserver.",
        method: GET,
        name: "get_protocols",
        path: "/_matrix/client/r0/thirdparty/protocols",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// Metadata about protocols supported by the homeserver.
        #[ruma_api(body)]
        pub protocols: HashMap<String, Protocol>,
    }
}
