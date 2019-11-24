//! Endpoints for room creation.

pub mod create_room;

use serde::{Deserialize, Serialize};

/// Whether or not a newly created room will be listed in the room directory.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Indicates that the room will be shown in the published room list.
    Public,
    /// Indicates that the room will not be shown in the published room list.
    Private,
}
