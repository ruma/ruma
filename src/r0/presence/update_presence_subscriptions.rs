//! [POST /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-presence-list-userid)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Update the presence subscriptions of the user.",
        method: POST,
        name: "update_presence_subscriptions",
        path: "/_matrix/client/r0/presence/list/:user_id",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// A list of user IDs to remove from the list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub drop: Vec<UserId>,
        /// A list of user IDs to add to the list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub invite: Vec<UserId>,
        /// The user whose presence state will be updated.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}
}
