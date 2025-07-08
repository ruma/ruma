//! Spaces endpoints.

use ruma_common::{
    room::RoomSummary,
    serde::{Raw, from_raw_json_value},
};
use ruma_events::space::child::HierarchySpaceChildEvent;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

pub mod get_hierarchy;

/// The summary of a parent space.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SpaceHierarchyParentSummary {
    /// The summary of the room.
    #[serde(flatten)]
    pub summary: RoomSummary,

    /// The stripped `m.space.child` events of the space.
    ///
    /// If the room is not a space, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildEvent>>,
}

impl SpaceHierarchyParentSummary {
    /// Construct a `SpaceHierarchyRoomsChunk` with the given summary and children state.
    pub fn new(summary: RoomSummary, children_state: Vec<Raw<HierarchySpaceChildEvent>>) -> Self {
        Self { summary, children_state }
    }
}

impl<'de> Deserialize<'de> for SpaceHierarchyParentSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SpaceHierarchyRoomsChunkDeHelper {
            children_state: Vec<Raw<HierarchySpaceChildEvent>>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let summary: RoomSummary = from_raw_json_value(&json)?;
        let SpaceHierarchyRoomsChunkDeHelper { children_state } = from_raw_json_value(&json)?;

        Ok(Self { summary, children_state })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::SpaceHierarchyParentSummary;

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

        let room = from_json_value::<SpaceHierarchyParentSummary>(json).unwrap();
        assert_eq!(room.summary.room_id, "!room:localhost");
        let space_child = room.children_state[0].deserialize().unwrap();
        assert_eq!(space_child.state_key, "!a:example.org");
    }
}
