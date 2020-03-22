//! [POST /_matrix/client/r0/user/{userId}/filter](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-user-userid-filter)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::FilterDefinition;

ruma_api! {
    metadata {
        description: "Create a new filter for event retrieval.",
        method: POST,
        name: "create_filter",
        path: "/_matrix/client/r0/user/:user_id/filter",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition,
        /// The ID of the user uploading the filter.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// The ID of the filter that was created.
        pub filter_id: String,
    }

    error: crate::Error
}
