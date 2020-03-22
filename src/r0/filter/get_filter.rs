//! [GET /_matrix/client/r0/user/{userId}/filter/{filterId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-user-userid-filter-filterid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::FilterDefinition;

ruma_api! {
    metadata {
        description: "Retrieve a previously created filter.",
        method: GET,
        name: "get_filter",
        path: "/_matrix/client/r0/user/:user_id/filter/:filter_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The ID of the filter to download.
        #[ruma_api(path)]
        pub filter_id: String,
        /// The user ID to download a filter for.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition,
    }

    error: crate::Error
}
