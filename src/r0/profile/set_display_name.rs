//! [PUT /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-profile-userid-displayname)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Set the display name of the user.",
        method: PUT,
        name: "set_display_name",
        path: "/_matrix/client/r0/profile/:user_id/displayname",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The new display name for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,
        /// The user whose display name will be set.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
