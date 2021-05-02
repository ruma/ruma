//! [GET /_matrix/client/r0/account/whoami](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-account-whoami)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Get information about the owner of a given access token.",
        method: GET,
        name: "whoami",
        path: "/_matrix/client/r0/account/whoami",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The id of the user that owns the access token.
        pub user_id: UserId,
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
    /// Creates a new `Response` with the given user ID.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}
