//! Endpoints for room aliases.

/// [PUT /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-directory-room-roomalias)
pub mod create_alias {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomAliasId, RoomId};

    ruma_api! {
        metadata {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room alias to set.
            #[ruma_api(path)]
            pub room_alias: RoomAliasId,
            /// The room ID to set.
            pub room_id: RoomId,
        }

        response {}
    }
}

/// [DELETE /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.2.0.html#delete-matrix-client-r0-directory-room-roomalias)
pub mod delete_alias {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomAliasId;

    ruma_api! {
        metadata {
            description: "Remove an alias from a room.",
            method: DELETE,
            name: "delete_alias",
            path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room alias to remove.
            #[ruma_api(path)]
            pub room_alias: RoomAliasId,
        }

        response {}
    }
}

/// [GET /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-directory-room-roomalias)
pub mod get_alias {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomAliasId, RoomId};

    ruma_api! {
        metadata {
            description: "Resolve a room alias to a room ID.",
            method: GET,
            name: "get_alias",
            path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room alias.
            #[ruma_api(path)]
            pub room_alias: RoomAliasId,
        }

        response {
            /// The room ID for this room alias.
            pub room_id: RoomId,
            /// A list of servers that are aware of this room ID.
            pub servers: Vec<String>,
        }
    }
}
