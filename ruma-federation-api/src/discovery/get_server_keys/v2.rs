//! [GET /_matrix/key/v2/server](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-key-v2-server-keyid)

use crate::discovery::ServerKey;
use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Gets the homeserver's published signing keys.",
        method: GET,
        name: "get_server_keys",
        path: "/_matrix/key/v2/server",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        /// Queried server key, signed by the notary server.
        #[ruma_api(body)]
        pub server_key: ServerKey,
    }
}
