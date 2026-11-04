//! Types for the [`m.marked_unread`] event.
//!
//! [`m.marked_unread`]: https://spec.matrix.org/v1.19/client-server-api/#unread-markers

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.marked_unread` event.
///
/// Whether the room has been explicitly marked as unread.
///
/// This event appears in the user's room account data for the room the marker is applicable for.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.marked_unread", kind = RoomAccountData)]
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

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};
    use strass::assert_let;

    use super::MarkedUnreadEventContent;
    use crate::{AnyRoomAccountDataEvent, RoomAccountDataEvent};

    #[test]
    fn deserialize() {
        let raw_marked_unread = json!({
            "type": "m.marked_unread",
            "content": {
                "unread": true,
            },
        });
        let marked_unread_account_data =
            from_json_value::<AnyRoomAccountDataEvent>(raw_marked_unread).unwrap();
        assert_let!(
            AnyRoomAccountDataEvent::MarkedUnread(marked_unread) = marked_unread_account_data
        );
        assert!(marked_unread.content.unread);
    }

    #[test]
    fn serialize() {
        let marked_unread = MarkedUnreadEventContent::new(true);
        let marked_unread_account_data = RoomAccountDataEvent { content: marked_unread };
        assert_to_canonical_json_eq!(
            marked_unread_account_data,
            json!({
                "type": "m.marked_unread",
                "content": {
                    "unread": true,
                },
            })
        );
    }
}
