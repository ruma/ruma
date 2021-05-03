//! [GET /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-profile-userid-displayname)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Get the display name of a user.",
        method: GET,
        name: "get_display_name",
        path: "/_matrix/client/r0/profile/:user_id/displayname",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The user whose display name will be retrieved.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {
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
    /// Creates a new `Response` with the given display name.
    pub fn new(displayname: Option<String>) -> Self {
        Self { displayname }
    }
}
