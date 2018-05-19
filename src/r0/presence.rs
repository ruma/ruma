//! Endpoints for user presence.

/// [PUT /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-presence-userid-status)
pub mod set_presence {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;
    use ruma_events::presence::PresenceState;

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
    }
}

/// [GET /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-presence-userid-status)
pub mod get_presence {
    use ruma_api_macros::ruma_api;
    use ruma_events::presence::PresenceState;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Get presence status for this user.",
            method: GET,
            name: "get_presence",
            path: "/_matrix/client/r0/presence/:user_id/status",
            rate_limited: false,
            requires_authentication: false,
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
            pub last_active_ago: Option<u64>,
            /// The user's presence state.
            pub presence: PresenceState,
        }
    }
}

/// [POST /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-presence-list-userid)
pub mod update_presence_subscriptions {
    use ruma_api_macros::ruma_api;
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
            #[serde(skip_serializing_if = "Vec::is_empty")]
            #[serde(default)]
            pub drop: Vec<UserId>,
            /// A list of user IDs to add to the list.
            #[serde(skip_serializing_if = "Vec::is_empty")]
            #[serde(default)]
            pub invite: Vec<UserId>,
            /// The user whose presence state will be updated.
            #[ruma_api(path)]
            pub user_id: UserId,
        }

        response {}
    }
}

/// [GET /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-presence-list-userid)
pub mod get_subscribed_presences {
    use ruma_api_macros::ruma_api;
    use ruma_events::presence::PresenceEvent;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Get the precence status from the user's subscriptions.",
            method: GET,
            name: "get_subscribed_presences",
            path: "/_matrix/client/r0/presence/list/:user_id",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The user whose presence state will be retrieved.
            #[ruma_api(path)]
            pub user_id: UserId,
        }

        response {
            /// A list of presence events for every user on this list.
            #[ruma_api(body)]
            presence_events: Vec<PresenceEvent>,
        }
    }
}
