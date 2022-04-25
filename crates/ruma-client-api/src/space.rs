//! Endpoints for spaces.
//!
//! See the [Matrix specification][spec] for more details about spaces.
//!
//! [spec]: https://spec.matrix.org/v1.2/client-server-api/#spaces

use js_int::UInt;
use ruma_common::{
    events::space::child::HierarchySpaceChildEvent,
    room::RoomType,
    serde::{Raw, StringEnum},
    OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, RoomName,
};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

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
    pub canonical_alias: Option<OwnedRoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<RoomName>>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: OwnedRoomId,

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
    pub avatar_url: Option<OwnedMxcUri>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: SpaceRoomJoinRule,

    /// The type of room from `m.room.create`, if any.
    pub room_type: Option<RoomType>,

    /// The stripped `m.space.child` events of the space-room.
    ///
    /// If the room is not a space-room, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildEvent>>,
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
    pub room_id: OwnedRoomId,

    /// Whether the room may be viewed by guest users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// The join rule of the room.
    pub join_rule: SpaceRoomJoinRule,

    /// The stripped `m.space.child` events of the space-room.
    ///
    /// If the room is not a space-room, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildEvent>>,
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

/// The rule used for users wishing to join a room.
///
/// In contrast to the regular [`JoinRule`](ruma_common::events::room::join_rules::JoinRule), this
/// enum does not hold the conditions for joining restricted rooms. Instead, the server is assumed
/// to only return rooms the user is allowed to join in a space hierarchy listing response.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum SpaceRoomJoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Users can join the room if they are invited, or they can request an invite to the room.
    ///
    /// They can be allowed (invited) or denied (kicked/banned) access.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Users can join the room if they are invited, or if they meet any of the conditions
    /// described in a set of [`AllowRule`](ruma_common::events::room::join_rules::AllowRule)s.
    ///
    /// These rules are not made available as part of a space hierarchy listing response and can
    /// only be seen by users inside the room.
    Restricted,

    /// Anyone can join the room without any prior action.
    Public,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Default for SpaceRoomJoinRule {
    fn default() -> Self {
        SpaceRoomJoinRule::Public
    }
}
