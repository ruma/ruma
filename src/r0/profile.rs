//! Endpoints for user profiles.

/// [GET /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-displayname)
pub mod get_display_name {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Get the display name of a user.",
            method: Method::Get,
            name: "get_display_name",
            path: "/_matrix/client/r0/profile/:user_id/displayname",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// The user whose display name will be retrieved.
            #[ruma_api(path)]
            pub user_id: UserId
        }

        response {
            /// The user's display name, if set.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>
        }
    }
}

/// [PUT /_matrix/client/r0/profile/{userId}/displayname](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-displayname)
pub mod set_display_name {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Set the display name of the user.",
            method: Method::Put,
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
    }
}

/// [GET /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid-avatar-url)
pub mod get_avatar_url {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Get the avatar URL of a user.",
            method: Method::Get,
            name: "get_avatar_url",
            path: "/_matrix/client/r0/profile/:user_id/avatar_url",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// The user whose avatar URL will be retrieved.
            #[ruma_api(path)]
            pub user_id: UserId
        }

        response {
            /// The user's avatar URL, if set.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub avatar_url: Option<String>
        }
    }
}

/// [PUT /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-profile-userid-avatar-url)
pub mod set_avatar_url {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Set the avatar URL of the user.",
            method: Method::Put,
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
    }
}

/// [GET /_matrix/client/r0/profile/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-profile-userid)
pub mod get_profile {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata {
            description: "Get all profile information of an user.",
            method: Method::Get,
            name: "get_profile",
            path: "/_matrix/client/r0/profile/:user_id",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// The user whose profile will be retrieved.
            #[ruma_api(path)]
            pub user_id: UserId
        }

        response {
            /// The user's avatar URL, if set.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub avatar_url: Option<String>,
            /// The user's display name, if set.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>
        }
    }
}
