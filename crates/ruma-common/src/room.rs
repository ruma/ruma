//! Common types for rooms.

use js_int::UInt;
use serde::{Deserialize, Serialize};

use crate::{
    serde::StringEnum, space::SpaceRoomJoinRule, EventEncryptionAlgorithm, OwnedMxcUri,
    OwnedRoomAliasId, OwnedRoomId, PrivOwnedStr, RoomVersionId,
};

/// An enum of possible room types.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum RoomType {
    /// Defines the room as a space.
    #[ruma_enum(rename = "m.space")]
    Space,

    /// Defines the room as a custom type.
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The summary of a room's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomSummary {
    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// The canonical alias of the room, if any.
    ///
    /// If the `compat-empty-string-null` cargo feature is enabled, this field being an empty
    /// string in JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub canonical_alias: Option<OwnedRoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The topic of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// The URL for the room's avatar, if one is set.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: SpaceRoomJoinRule,

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub allowed_room_ids: Vec<OwnedRoomId>,

    /// Whether the room may be viewed by users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// If the room is encrypted, the algorithm used for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EventEncryptionAlgorithm>,

    /// The version of the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_version: Option<RoomVersionId>,
}

impl RoomSummary {
    /// Construct a new `RoomSummary` with the given required fields.
    pub fn new(
        room_id: OwnedRoomId,
        join_rule: SpaceRoomJoinRule,
        guest_can_join: bool,
        num_joined_members: UInt,
        world_readable: bool,
    ) -> Self {
        Self {
            room_id,
            canonical_alias: None,
            name: None,
            topic: None,
            avatar_url: None,
            room_type: None,
            num_joined_members,
            join_rule,
            allowed_room_ids: Vec::new(),
            world_readable,
            guest_can_join,
            encryption: None,
            room_version: None,
        }
    }
}
