//! [PUT /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-profile-userid-displayname)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Set the display name of the user.",
        method: PUT,
        name: "set_display_name",
        path: "/_matrix/client/r0/profile/:user_id/displayname",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The user whose display name will be set.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The new display name for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<&'a str>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and display name.
    pub fn new(user_id: &'a UserId, displayname: Option<&'a str>) -> Self {
        Self { user_id, displayname }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
