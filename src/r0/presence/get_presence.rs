//! [GET /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-presence-userid-status)

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_events::presence::PresenceState;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Get presence status for this user.",
        method: GET,
        name: "get_presence",
        path: "/_matrix/client/r0/presence/:user_id/status",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The user whose presence state will be retrieved.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// The state message for this user if one was set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,
        /// Whether or not the user is currently active.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub currently_active: Option<bool>,
        /// The length of time in milliseconds since an action was performed by the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::serde::duration::opt_ms")]
        pub last_active_ago: Option<Duration>,
        /// The user's presence state.
        pub presence: PresenceState,
    }

    error: crate::Error
}
