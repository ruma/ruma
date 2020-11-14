//! [POST /_matrix/identity/v2/account/logout](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-account-logout)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Logs out the access token, preventing it from being used to authenticate future requests to the server.",
        method: POST,
        name: "logout",
        path: "/_matrix/identity/v2/account/logout",
        authentication: AccessToken,
        rate_limited: false,
    }

    #[derive(Default)]
    request: {}

    #[derive(Default)]
    response: {}
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
