//! Endpoints for spaces.

use js_int::UInt;
use ruma_common::{
    directory::PublicRoomJoinRule, events::space::child::HierarchySpaceChildStateEvent,
    room::RoomType, serde::Raw, MxcUri, RoomAliasId, RoomId, RoomName,
};
use serde::{Deserialize, Serialize};

pub mod get_hierarchy;

/// A chunk of a space hierarchy response, describing one room.
///
/// To create an instance of this type, first create a `SpaceHierarchyRoomsChunkInit` and convert it
/// via `SpaceHierarchyRoomsChunk::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SpaceHierarchyRoomsChunk {
    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub canonical_alias: Option<Box<RoomAliasId>>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<RoomName>>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: Box<RoomId>,

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
    ///
    /// If you activate the `compat` feature, this field being an empty string in JSON will result
    /// in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<Box<MxcUri>>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: PublicRoomJoinRule,

    /// The type of room from `m.room.create`, if any.
    pub room_type: Option<RoomType>,

    /// The stripped `m.space.child` events of the space-room.
    ///
    /// If the room is not a space-room, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildStateEvent>>,
}

/// Initial set of mandatory fields of `SpaceHierarchyRoomsChunk`.
///
/// This struct will not be updated even if additional fields are added to
/// `SpaceHierarchyRoomsChunk` in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SpaceHierarchyRoomsChunkInit {
    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: Box<RoomId>,

    /// Whether the room may be viewed by guest users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// The join rule of the room.
    pub join_rule: PublicRoomJoinRule,

    /// The stripped `m.space.child` events of the space-room.
    ///
    /// If the room is not a space-room, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildStateEvent>>,
}

impl From<SpaceHierarchyRoomsChunkInit> for SpaceHierarchyRoomsChunk {
    fn from(init: SpaceHierarchyRoomsChunkInit) -> Self {
        let SpaceHierarchyRoomsChunkInit {
            num_joined_members,
            room_id,
            world_readable,
            guest_can_join,
            join_rule,
            children_state,
        } = init;

        Self {
            canonical_alias: None,
            name: None,
            num_joined_members,
            room_id,
            topic: None,
            world_readable,
            guest_can_join,
            avatar_url: None,
            join_rule,
            room_type: None,
            children_state,
        }
    }
}
