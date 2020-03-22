//! [PUT /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-presence-userid-status)

use ruma_api::ruma_api;
use ruma_events::presence::PresenceState;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Set presence status for this user.",
        method: PUT,
        name: "set_presence",
        path: "/_matrix/client/r0/presence/:user_id/status",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The new presence state.
        pub presence: PresenceState,
        /// The status message to attach to this state.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,
        /// The user whose presence state will be updated.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
