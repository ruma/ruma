//! Endpoints for room creation.

/// [POST /_matrix/client/r0/createRoom](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-createroom)
pub mod create_room {
    use ruma_identifiers::{RoomId, UserId};
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Create a new room.",
            method: POST,
            name: "create_room",
            path: "/_matrix/client/r0/createRoom",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// Extra keys to be added to the content of the `m.room.create`.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub creation_content: Option<CreationContent>,
            /// A list of user IDs to invite to the room.
            ///
            /// This will tell the server to invite everyone in the list to the newly created room.
            #[serde(skip_serializing_if = "Vec::is_empty")]
            #[serde(default)]
            pub invite: Vec<UserId>,
            /// If this is included, an `m.room.name` event will be sent into the room to indicate
            /// the name of the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub name: Option<String>,
            /// Convenience parameter for setting various default state events based on a preset.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub preset: Option<RoomPreset>,
            /// The desired room alias local part.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_alias_name: Option<String>,
            /// If this is included, an `m.room.topic` event will be sent into the room to indicate
            /// the topic for the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub topic: Option<String>,
            /// A public visibility indicates that the room will be shown in the published room
            /// list. A private visibility will hide the room from the published room list. Rooms
            /// default to private visibility if this key is not included.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub visibility: Option<Visibility>,
            // TODO: missing `invite_3pid`, `initial_state`
        }

        response {
            /// The created room's ID.
            pub room_id: RoomId,
        }
    }

    /// Extra options to be added to the `m.room.create` event.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CreationContent {
        /// Whether users on other servers can join this room.
        ///
        /// Defaults to `true` if key does not exist.
        #[serde(rename="m.federate")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub federate: Option<bool>,
    }

    /// A convenience parameter for setting a few default state events.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum RoomPreset {
        /// `join_rules` is set to `invite` and `history_visibility` is set to `shared`.
        #[serde(rename="private_chat")]
        PrivateChat,
        /// `join_rules` is set to `public` and `history_visibility` is set to `shared`.
        #[serde(rename="public_chat")]
        PublicChat,
        /// Same as `PrivateChat`, but all initial invitees get the same power level as the creator.
        #[serde(rename="trusted_private_chat")]
        TrustedPrivateChat,
    }

    /// Whether or not a newly created room will be listed in the room directory.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum Visibility {
        /// Indicates that the room will be shown in the published room list.
        #[serde(rename = "public")]
        Public,
        /// Indicates that the room from the published room list.
        #[serde(rename = "private")]
        Private,
    }
}
