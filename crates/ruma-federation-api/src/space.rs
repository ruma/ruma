//! Spaces endpoints.

use ruma_common::{room::RoomSummary, serde::Raw};
use ruma_events::space::child::HierarchySpaceChildEvent;
use serde::{Deserialize, Serialize};

pub mod get_hierarchy;

/// The summary of a parent space.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
