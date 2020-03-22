//! [GET /_matrix/client/r0/user/{userId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-account-data-type)

use ruma_api::ruma_api;
use ruma_events::{collections::only, EventResult};
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Gets global account data for a user.",
        name: "get_global_account_data",
        method: GET,
        path: "/_matrix/client/r0/user/:user_id/account_data/:event_type",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// User ID of user for whom to retrieve data.
        #[ruma_api(path)]
        pub user_id: UserId,
        /// Type of data to retrieve.
        #[ruma_api(path)]
        pub event_type: String,
    }

    response {
        /// Account data content for the given type.
        #[ruma_api(body)]
        #[wrap_incoming(with EventResult)]
        pub account_data: only::Event,
    }

    error: crate::Error
}
