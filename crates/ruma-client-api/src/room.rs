//! Endpoints for room management.

pub mod aliases;
pub mod create_room;
pub mod get_event_by_timestamp;
pub mod get_room_event;
#[cfg(feature = "unstable-msc3266")]
pub mod get_summary;
pub mod report_content;
pub mod upgrade_room;

use ruma_common::serde::StringEnum;

use crate::PrivOwnedStr;

/// Whether or not a newly created room will be listed in the room directory.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Visibility {
    /// Indicates that the room will be shown in the published room list.
    Public,

    /// Indicates that the room will not be shown in the published room list.
    #[default]
    Private,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
