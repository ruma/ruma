//! [GET /_matrix/client/r0/thirdparty/protocols](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-protocols)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::thirdparty::Protocol;

ruma_api! {
    metadata: {
        description: "Fetches the overall metadata about protocols supported by the homeserver.",
        method: GET,
        name: "get_protocols",
        path: "/_matrix/client/r0/thirdparty/protocols",
        rate_limited: false,
        authentication: AccessToken,
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
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given procotols.
    pub fn new(protocols: BTreeMap<String, Protocol>) -> Self {
        Self { protocols }
    }
}
