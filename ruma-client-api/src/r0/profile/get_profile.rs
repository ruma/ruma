//! [GET /_matrix/client/r0/profile/{userId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-profile-userid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Get all profile information of an user.",
        method: GET,
        name: "get_profile",
        path: "/_matrix/client/r0/profile/:user_id",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {
        /// The user's avatar URL, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,

        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given avatar URL and display name.
    pub fn new(avatar_url: Option<String>, displayname: Option<String>) -> Self {
        Self { avatar_url, displayname }
    }
}
