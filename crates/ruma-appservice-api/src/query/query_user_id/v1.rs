//! [GET /_matrix/app/v1/users/{userId}](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-users-userid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "This endpoint is invoked by the homeserver on an application service to query the existence of a given user ID.",
        method: GET,
        name: "query_user_id",
        path: "/_matrix/app/v1/users/:user_id",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The user ID being queried.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user id.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
