//! Endpoints for spaces.
//!
//! See the [Matrix specification][spec] for more details about spaces.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#spaces

use js_int::UInt;
use ruma_common::{
    room::RoomType, serde::Raw, space::SpaceRoomJoinRule, EventEncryptionAlgorithm, OwnedMxcUri,
    OwnedRoomAliasId, OwnedRoomId, RoomVersionId,
};
use ruma_events::space::child::HierarchySpaceChildEvent;
use serde::{Deserialize, Serialize};

pub mod get_hierarchy;

/// A chunk of a space hierarchy response, describing one room.
///
/// To create an instance of this type, first create a `SpaceHierarchyRoomsChunkInit` and convert it
/// via `SpaceHierarchyRoomsChunk::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SpaceHierarchyRoomsChunk {
    /// The canonical alias of the room, if any.
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

    /// If the room is a restricted room, these are the room IDs which are specified by the join
    /// rules.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub allowed_room_ids: Vec<OwnedRoomId>,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,

    /// If the room is encrypted, the algorithm used for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EventEncryptionAlgorithm>,

    /// The version of the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_version: Option<RoomVersionId>,

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
            allowed_room_ids: Vec::new(),
            room_type: None,
            encryption: None,
            room_version: None,
            children_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::SpaceHierarchyRoomsChunk;

    #[test]
    fn deserialize_space_hierarchy_rooms_chunk() {
        let json = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
            "join_rule": "restricted",
            "allowed_room_ids": ["!otherroom:localhost"],
            "children_state": [
                {
                    "content": {
                        "via": [
                            "example.org"
                        ]
                    },
                    "origin_server_ts": 1_629_413_349,
                    "sender": "@alice:example.org",
                    "state_key": "!a:example.org",
                    "type": "m.space.child"
                }
            ],
        });

        let room = from_json_value::<SpaceHierarchyRoomsChunk>(json).unwrap();
        assert_eq!(room.room_id, "!room:localhost");
        let space_child = room.children_state[0].deserialize().unwrap();
        assert_eq!(space_child.state_key, "!a:example.org");
    }
}
