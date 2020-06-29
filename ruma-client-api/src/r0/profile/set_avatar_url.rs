//! [PUT /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-profile-userid-avatar-url)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Set the avatar URL of the user.",
        method: PUT,
        name: "set_avatar_url",
        path: "/_matrix/client/r0/profile/:user_id/avatar_url",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The user whose avatar URL will be set.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// The new avatar URL for the user.
        ///
        /// `None` is used to unset the avatar.
        pub avatar_url: Option<String>,
    }

    response: {}

    error: crate::Error
}
