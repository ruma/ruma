//! [GET /_matrix/client/r0/capabilities](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-capabilities)

use ruma_api::ruma_api;

use super::Capabilities;

ruma_api! {
    metadata: {
        description: "Gets information about the server's supported feature set and other relevant capabilities.",
        method: GET,
        name: "get_capabilities",
        r0: "/_matrix/client/r0/capabilities",
        stable: "/_matrix/client/v3/capabilities",
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
