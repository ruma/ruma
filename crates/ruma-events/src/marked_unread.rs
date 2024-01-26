//! Types for the [`m.marked_unread`] event.
//!
//! [`m.marked_unread`]: https://github.com/matrix-org/matrix-spec-proposals/pull/2867

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.marked_unread` event.
///
/// Whether the room has been explicitly marked as unread.
///
/// This event appears in the user's room account data for the room the marker is applicable for.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "com.famedly.marked_unread", kind = RoomAccountData)]
pub struct MarkedUnreadEventContent {
    /// The current unread state.
    pub unread: bool,
}

impl MarkedUnreadEventContent {
    /// Creates a new `MarkedUnreadEventContent` with the given value.
    pub fn new(unread: bool) -> Self {
        Self { unread }
    }
}
