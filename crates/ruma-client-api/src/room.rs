//! Endpoints for room management.

pub mod aliases;
pub mod create_room;
pub mod get_room_event;
pub mod report_content;
pub mod upgrade_room;

use ruma_serde::StringEnum;

use crate::PrivOwnedStr;

/// Whether or not a newly created room will be listed in the room directory.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Visibility {
    /// Indicates that the room will be shown in the published room list.
    Public,

    /// Indicates that the room will not be shown in the published room list.
    Private,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Visibility {
    /// Creates a string slice from this `Visibility`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}
