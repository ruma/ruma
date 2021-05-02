//! [GET /_matrix/app/v1/thirdparty/user](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-thirdparty-user)

use ruma_api::ruma_api;
use ruma_common::thirdparty::User;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Retrieve an array of third party users from a Matrix User ID.",
        method: GET,
        name: "get_user_for_user_id",
        path: "/_matrix/app/v1/thirdparty/user",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The Matrix User ID to look up.
        #[ruma_api(query)]
        pub userid: &'a UserId,
    }

    response: {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user id.
    pub fn new(userid: &'a UserId) -> Self {
        Self { userid }
    }
}

impl Response {
    /// Creates a new `Response` with the given users.
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }
}
