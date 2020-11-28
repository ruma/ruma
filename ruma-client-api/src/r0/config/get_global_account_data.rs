//! [GET /_matrix/client/r0/user/{userId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-account-data-type)

use ruma_api::ruma_api;
use ruma_events::AnyBasicEvent;
use ruma_identifiers::UserId;
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Gets global account data for a user.",
        name: "get_global_account_data",
        method: GET,
        path: "/_matrix/client/r0/user/:user_id/account_data/:event_type",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// User ID of user for whom to retrieve data.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// Type of data to retrieve.
        #[ruma_api(path)]
        pub event_type: &'a str,
    }

    response: {
        /// Account data content for the given type.
        #[ruma_api(body)]
        pub account_data: Raw<AnyBasicEvent>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and event type.
    pub fn new(user_id: &'a UserId, event_type: &'a str) -> Self {
        Self { user_id, event_type }
    }
}

impl Response {
    /// Creates a new `Response` with the given account data.
    pub fn new(account_data: Raw<AnyBasicEvent>) -> Self {
        Self { account_data }
    }
}
