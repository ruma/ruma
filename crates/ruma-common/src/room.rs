//! Common types for rooms.

use crate::{serde::StringEnum, PrivOwnedStr};

/// An enum of possible room types.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
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
