//! Endpoints for room management.

pub mod create_room;
pub mod get_room_event;
pub mod report_content;
pub mod upgrade_room;

use ruma_serde::StringEnum;

/// Whether or not a newly created room will be listed in the room directory.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum Visibility {
    /// Indicates that the room will be shown in the published room list.
    Public,

    /// Indicates that the room will not be shown in the published room list.
    Private,

    #[doc(hidden)]
    _Custom(String),
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}
