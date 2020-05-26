//! Endpoints for the public room directory.

pub mod get_public_rooms;
pub mod get_public_rooms_filtered;
pub mod get_room_visibility;
pub mod set_room_visibility;

use js_int::UInt;
use ruma_identifiers::{RoomAliasId, RoomId};
use serde::{Deserialize, Serialize};

/// A chunk of a room list response, describing one room
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicRoomsChunk {
    /// Aliases of the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<RoomAliasId>,

    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_alias: Option<RoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: RoomId,

    /// The topic of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Whether the room may be viewed by guest users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// The URL for the room's avatar, if one is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}
