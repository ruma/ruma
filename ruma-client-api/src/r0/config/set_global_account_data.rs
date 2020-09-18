//! [PUT /_matrix/client/r0/user/{userId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-user-userid-account-data-type)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Sets global account data.",
        method: PUT,
        name: "set_global_account_data",
        path: "/_matrix/client/r0/user/:user_id/account_data/:event_type",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// Arbitrary JSON to store as config data.
        ///
        /// To create a `Box<RawJsonValue>`, use `serde_json::value::to_raw_value`.
        #[ruma_api(body)]
        pub data: Box<RawJsonValue>,

        /// The event type of the account_data to set.
        ///
        /// Custom types should be namespaced to avoid clashes.
        #[ruma_api(path)]
        pub event_type: &'a str,

        /// The ID of the user to set account_data for.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given data, event type and user ID.
    pub fn new(data: Box<RawJsonValue>, event_type: &'a str, user_id: &'a UserId) -> Self {
        Self { data, event_type, user_id }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
