//! [PUT /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-profile-userid-avatar-url)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Set the avatar URL of the user.",
        method: PUT,
        name: "set_avatar_url",
        path: "/_matrix/client/r0/profile/:user_id/avatar_url",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The new avatar URL for the user.
        pub avatar_url: String,
        /// The user whose avatar URL will be set.
        #[ruma_api(path)]
        pub user_id: UserId
    }

    response {}

    error: crate::Error
}
