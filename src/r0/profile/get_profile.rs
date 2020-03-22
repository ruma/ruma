//! [GET /_matrix/client/r0/profile/{userId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-profile-userid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Get all profile information of an user.",
        method: GET,
        name: "get_profile",
        path: "/_matrix/client/r0/profile/:user_id",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// The user's avatar URL, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,
        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,
    }

    error: crate::Error
}
