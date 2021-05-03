//! [GET /_matrix/client/r0/user/{userId}/filter/{filterId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-filter-filterid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::IncomingFilterDefinition;

ruma_api! {
    metadata: {
        description: "Retrieve a previously created filter.",
        method: GET,
        name: "get_filter",
        path: "/_matrix/client/r0/user/:user_id/filter/:filter_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The user ID to download a filter for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The ID of the filter to download.
        #[ruma_api(path)]
        pub filter_id: &'a str,
    }

    response: {
        /// The filter definition.
        #[ruma_api(body)]
        pub filter: IncomingFilterDefinition,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and filter ID.
    pub fn new(user_id: &'a UserId, filter_id: &'a str) -> Self {
        Self { user_id, filter_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given filter definition.
    pub fn new(filter: IncomingFilterDefinition) -> Self {
        Self { filter }
    }
}
