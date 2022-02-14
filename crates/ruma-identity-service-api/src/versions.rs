//! `GET /_matrix/identity/versions` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityversions

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get the versions of the identity service API supported by this endpoint.",
        method: GET,
        name: "versions",
        // note: The MSC does not specify an unstable endpoint, nor does it offer any fallbacks,
        // regardless, this is only supported after v1.1, so a client has to take a gamble if an
        // identity service supports this or not.
        //
        // Safe to say, if the endpoint returns a 404, then it's likely only supporting < v1.1.
        stable_path: "/_matrix/identity/versions",
        rate_limited: false,
        authentication: None,
        added: 1.1,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of Matrix client API protocol versions supported by the endpoint.
        pub versions: Vec<String>,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given `versions`.
    pub fn new(versions: Vec<String>) -> Self {
        Self { versions }
    }
}
