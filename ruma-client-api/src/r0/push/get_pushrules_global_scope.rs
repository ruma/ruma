//! [GET /_matrix/client/r0/pushrules/global/](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-pushrules)

use ruma_api::ruma_api;
use ruma_common::push::Ruleset;

ruma_api! {
    metadata: {
        description: "Retrieve all push rulesets in the global scope for this user.",
        method: GET,
        name: "get_pushrules_global_scope",
        path: "/_matrix/client/r0/pushrules/global/",
        rate_limited: false,
        requires_authentication: true,
    }

    #[derive(Default)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {}

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {
        /// The global ruleset.
        #[ruma_api(body)]
        pub global: Ruleset,
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
    /// Creates a new `Response` with the given global ruleset.
    pub fn new(global: Ruleset) -> Self {
        Self { global }
    }
}
