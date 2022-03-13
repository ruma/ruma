//! `/v3/` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3capabilities

use ruma_common::api::ruma_api;

use super::Capabilities;

ruma_api! {
    metadata: {
        description: "Gets information about the server's supported feature set and other relevant capabilities.",
        method: GET,
        name: "get_capabilities",
        r0_path: "/_matrix/client/r0/capabilities",
        stable_path: "/_matrix/client/v3/capabilities",
        rate_limited: true,
        authentication: AccessToken,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The capabilities the server supports
        pub capabilities: Capabilities,
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
    /// Creates a new `Response` with the given capabilities.
    pub fn new(capabilities: Capabilities) -> Self {
        Self { capabilities }
    }
}

impl From<Capabilities> for Response {
    fn from(capabilities: Capabilities) -> Self {
        Self::new(capabilities)
    }
}
