//! `/v3/` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3capabilities

use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};

use super::Capabilities;

const METADATA: Metadata = metadata! {
    description: "Gets information about the server's supported feature set and other relevant capabilities.",
    method: GET,
    name: "get_capabilities",
    rate_limited: true,
    authentication: AccessToken,
    history: {
        1.0 => "/_matrix/client/r0/capabilities",
        1.1 => "/_matrix/client/v3/capabilities",
    }
};

#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {}

#[response(error = crate::Error)]
pub struct Response {
    /// The capabilities the server supports
    pub capabilities: Capabilities,
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
