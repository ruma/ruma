//! [GET /_matrix/client/r0/thirdparty/user](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-user)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::User;

ruma_api! {
    metadata {
        description: "Retrieve an array of third party users from a Matrix User ID.",
        method: GET,
        name: "get_user_for_user_id",
        path: "/_matrix/client/r0/thirdparty/user",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The Matrix User ID to look up.
        #[ruma_api(query)]
        pub userid: UserId,
    }

    response {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }

    error: crate::Error
}
