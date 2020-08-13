//! [POST /_matrix/client/r0/user/{userId}/filter](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-user-userid-filter)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::{FilterDefinition, IncomingFilterDefinition};

ruma_api! {
    metadata: {
        description: "Create a new filter for event retrieval.",
        method: POST,
        name: "create_filter",
        path: "/_matrix/client/r0/user/:user_id/filter",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The ID of the user uploading the filter.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition<'a>,
    }

    response: {
        /// The ID of the filter that was created.
        pub filter_id: String,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and filter definition.
    pub fn new(user_id: &'a UserId, filter: FilterDefinition<'a>) -> Self {
        Self { user_id, filter }
    }
}

impl Response {
    /// Creates a new `Response` with the given filter ID.
    pub fn new(filter_id: String) -> Self {
        Self { filter_id }
    }
}
