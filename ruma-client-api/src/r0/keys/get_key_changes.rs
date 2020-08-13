//! [GET /_matrix/client/r0/keys/changes](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-keys-changes)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Gets a list of users who have updated their device identity keys since a previous sync token.",
        method: GET,
        name: "get_key_changes",
        path: "/_matrix/client/r0/keys/changes",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The desired start point of the list.
        ///
        /// Should be the next_batch field from a response to an earlier call to /sync.
        #[ruma_api(query)]
        pub from: &'a str,

        /// The desired end point of the list.
        ///
        /// Should be the next_batch field from a recent call to /sync - typically the most recent
        /// such call.
        #[ruma_api(query)]
        pub to: &'a str,
    }

    response: {
        /// The Matrix User IDs of all users who updated their device identity keys.
        pub changed: Vec<UserId>,

        /// The Matrix User IDs of all users who may have left all the end-to-end
        /// encrypted rooms they previously shared with the user.
        pub left: Vec<UserId>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given start and end points.
    pub fn new(from: &'a str, to: &'a str) -> Self {
        Self { from, to }
    }
}

impl Response {
    /// Creates a new `Response` with the given changed and left user ID lists.
    pub fn new(changed: Vec<UserId>, left: Vec<UserId>) -> Self {
        Self { changed, left }
    }
}
