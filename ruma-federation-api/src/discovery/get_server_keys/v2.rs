//! [GET /_matrix/key/v2/server](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-key-v2-server-keyid)

use crate::discovery::SigningKey;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Gets the homeserver's published signing keys.",
        method: GET,
        name: "get_server_keys",
        path: "/_matrix/key/v2/server",
        rate_limited: false,
        authentication: None,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// Queried server key, signed by the notary server.
        #[ruma_api(body)]
        pub server_key: SigningKey,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given server key.
    pub fn new(server_key: SigningKey) -> Self {
        Self { server_key }
    }
}

impl From<SigningKey> for Response {
    fn from(server_key: SigningKey) -> Self {
        Self::new(server_key)
    }
}
