//! Common types for rooms.

use ruma_serde::StringEnum;

use crate::PrivOwnedStr;

/// An enum of possible room types.
///
/// This type can hold an arbitrary string. To check for room types that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum RoomType {
    /// Defines the room as a space.
    #[ruma_enum(rename = "m.space")]
    Space,

    /// Defines the room as a custom type.
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl RoomType {
    /// Creates a string slice from this `RoomType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
