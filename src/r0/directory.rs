//! Endpoints for the public room directory.

/// [GET /_matrix/client/r0/publicRooms](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-publicrooms)
pub mod get_public_rooms {
    use ruma_identifiers::{RoomId, RoomAliasId};
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Get the list of rooms in this homeserver's public directory.",
            method: GET,
            name: "get_public_rooms",
            path: "/_matrix/client/r0/publicRooms",
            rate_limited: false,
            requires_authentication: false,
        }

        request {}

        response {
            /// A pagination token for the response.
            pub start: String,
            /// A paginated chunk of public rooms.
            pub chunk: Vec<PublicRoomsChunk>,
            /// A pagination token for the response.
            pub end: String
        }
    }

    /// A chunk of the response, describing one room
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PublicRoomsChunk {
        /// Aliases of the room.
        //#[serde(skip_serializing_if = "Option::is_none")]
        pub aliases: Option<Vec<RoomAliasId>>,
        /// The URL for the room's avatar, if one is set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,
        /// Whether guest users may join the room and participate in it.
        ///
        /// If they can, they will be subject to ordinary power level rules like any other user.
        pub guest_can_join: bool,
        /// The name of the room, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// The number of members joined to the room.
        pub num_joined_members: u64,
        /// The ID of the room.
        pub room_id: RoomId,
        /// The topic of the room, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,
        /// Whether the room may be viewed by guest users without joining.
        pub world_readable: bool,
    }
}
