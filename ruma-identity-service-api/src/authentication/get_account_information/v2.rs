//! [GET /_matrix/identity/v2/account](https://matrix.org/docs/spec/identity_service/latest#get-matrix-identity-v2-account)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Gets information about what user owns the access token used in the request.",
        method: POST,
        name: "get_account_information",
        path: "/_matrix/identity/v2/account",
        authentication: AccessToken,
        rate_limited: false,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The user ID which registered the token.
        pub user_id: UserId,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given `UserId`.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}
