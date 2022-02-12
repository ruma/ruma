//! [GET /_matrix/identity/v2](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Checks that an identity server is available at this API endpoint.",
        method: GET,
        name: "status",
        stable_path: "/_matrix/identity/v2",
        authentication: None,
        rate_limited: false,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    #[derive(Default)]
    response: {}
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
