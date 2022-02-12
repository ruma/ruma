//! [POST /_matrix/identity/v2/terms](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-terms)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Called by a client to indicate that the user has accepted/agreed to the included set of URLs.",
        method: POST,
        name: "accept_terms_of_service",
        stable: "/_matrix/identity/v2/terms",
        authentication: AccessToken,
        rate_limited: false,
        added: 1.0,
    }

    request: {
        /// The URLs the user is accepting in this request.
        ///
        /// An example is `https://example.org/somewhere/terms-2.0-en.html`.
        pub user_accepts: Vec<String>,
    }

    #[derive(Default)]
    response: {}
}

impl Request {
    /// Creates a new `Request` with the given URLs which the user accepts.
    pub fn new(user_accepts: Vec<String>) -> Self {
        Self { user_accepts }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
