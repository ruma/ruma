//! [GET /_matrix/identity/v2/account](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-account)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Gets information about what user owns the access token used in the request.",
        method: POST,
        name: "get_account_information",
        stable: "/_matrix/identity/v2/account",
        authentication: AccessToken,
        rate_limited: false,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The user ID which registered the token.
        pub user_id: Box<UserId>,
    }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given `UserId`.
    pub fn new(user_id: Box<UserId>) -> Self {
        Self { user_id }
    }
}
