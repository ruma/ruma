//! Endpoints for spaces.
//!
//! See the [Matrix specification][spec] for more details about spaces.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#spaces

use ruma_common::{room::RoomSummary, serde::Raw};
use ruma_events::space::child::HierarchySpaceChildEvent;
use serde::{Deserialize, Serialize};

pub mod get_hierarchy;

/// A chunk of a space hierarchy response, describing one room.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SpaceHierarchyRoomsChunk {
    /// The summary of the room.
    #[serde(flatten)]
    pub summary: RoomSummary,

    /// The stripped `m.space.child` events of the space.
    ///
    /// If the room is not a space, this should be empty.
    pub children_state: Vec<Raw<HierarchySpaceChildEvent>>,
}

impl SpaceHierarchyRoomsChunk {
    /// Construct a `SpaceHierarchyRoomsChunk` with the given summary and children state.
    pub fn new(summary: RoomSummary, children_state: Vec<Raw<HierarchySpaceChildEvent>>) -> Self {
        Self { summary, children_state }
    }
}
