//! [GET /_matrix/client/r0/thirdparty/protocols](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-thirdparty-protocols)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::thirdparty::Protocol;

ruma_api! {
    metadata: {
        description: "Fetches the overall metadata about protocols supported by the homeserver.",
        method: GET,
        name: "get_protocols",
        r0: "/_matrix/client/r0/thirdparty/protocols",
        stable: "/_matrix/client/v3/thirdparty/protocols",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// Metadata about protocols supported by the homeserver.
        #[ruma_api(body)]
        pub protocols: BTreeMap<String, Protocol>,
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
    /// Creates a new `Response` with the given protocols.
    pub fn new(protocols: BTreeMap<String, Protocol>) -> Self {
        Self { protocols }
    }
}
