//! Spaces endpoints.

use js_int::UInt;
use ruma_common::{
    room::RoomType, serde::Raw, space::SpaceRoomJoinRule, OwnedMxcUri, OwnedRoomAliasId,
    OwnedRoomId,
};
use ruma_events::space::child::HierarchySpaceChildEvent;
use serde::{Deserialize, Serialize};

pub mod get_hierarchy;

/// The summary of a parent space.
///
/// To create an instance of this type, first create a `SpaceHierarchyParentSummaryInit` and convert
/// it via `SpaceHierarchyParentSummary::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SpaceHierarchyParentSummary {
    /// The canonical alias of the room, if any.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty
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
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: SpaceRoomJoinRule,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,

    /// The stripped `m.space.child` events of the space-room.
    ///
    /// If the room is not a space-room, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildEvent>>,

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub allowed_room_ids: Vec<OwnedRoomId>,
}

/// Initial set of mandatory fields of `SpaceHierarchyParentSummary`.
///
/// This struct will not be updated even if additional fields are added to
/// `SpaceHierarchyParentSummary` in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SpaceHierarchyParentSummaryInit {
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

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    pub allowed_room_ids: Vec<OwnedRoomId>,
}

impl From<SpaceHierarchyParentSummaryInit> for SpaceHierarchyParentSummary {
    fn from(init: SpaceHierarchyParentSummaryInit) -> Self {
        let SpaceHierarchyParentSummaryInit {
            num_joined_members,
            room_id,
            world_readable,
            guest_can_join,
            join_rule,
            children_state,
            allowed_room_ids,
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
            allowed_room_ids,
        }
    }
}

/// The summary of a space's child.
///
/// To create an instance of this type, first create a `SpaceHierarchyChildSummaryInit` and convert
/// it via `SpaceHierarchyChildSummary::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SpaceHierarchyChildSummary {
    /// The canonical alias of the room, if any.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub canonical_alias: Option<OwnedRoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

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
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: SpaceRoomJoinRule,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub allowed_room_ids: Vec<OwnedRoomId>,
}

/// Initial set of mandatory fields of `SpaceHierarchyChildSummary`.
///
/// This struct will not be updated even if additional fields are added to
/// `SpaceHierarchyChildSummary` in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SpaceHierarchyChildSummaryInit {
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

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    pub allowed_room_ids: Vec<OwnedRoomId>,
}

impl From<SpaceHierarchyChildSummaryInit> for SpaceHierarchyChildSummary {
    fn from(init: SpaceHierarchyChildSummaryInit) -> Self {
        let SpaceHierarchyChildSummaryInit {
            num_joined_members,
            room_id,
            world_readable,
            guest_can_join,
            join_rule,
            allowed_room_ids,
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
            allowed_room_ids,
        }
    }
}

impl From<SpaceHierarchyParentSummary> for SpaceHierarchyChildSummary {
    fn from(parent: SpaceHierarchyParentSummary) -> Self {
        let SpaceHierarchyParentSummary {
            canonical_alias,
            name,
            num_joined_members,
            room_id,
            topic,
            world_readable,
            guest_can_join,
            avatar_url,
            join_rule,
            room_type,
            children_state: _,
            allowed_room_ids,
        } = parent;

        Self {
            canonical_alias,
            name,
            num_joined_members,
            room_id,
            topic,
            world_readable,
            guest_can_join,
            avatar_url,
            join_rule,
            room_type,
            allowed_room_ids,
        }
    }
}
