//! Types for the [`m.marked_unread`] event.
//!
//! [`m.marked_unread`]: https://spec.matrix.org/latest/client-server-api/#unread-markers

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.marked_unread` event.
///
/// Whether the room has been explicitly marked as unread.
///
/// This event appears in the user's room account data for the room the marker is applicable for.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

/// The content of a [`com.famedly.marked_unread`] event, the unstable version of
/// [MarkedUnreadEventContent].
///
/// Whether the room has been explicitly marked as unread.
///
/// This event appears in the user's room account data for the room the marker is applicable for.
///
/// [`com.famedly.marked_unread`]: https://github.com/matrix-org/matrix-spec-proposals/pull/2867
#[cfg(feature = "unstable-msc2867")]
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "com.famedly.marked_unread", kind = RoomAccountData)]
#[serde(transparent)]
pub struct UnstableMarkedUnreadEventContent(pub MarkedUnreadEventContent);

#[cfg(feature = "unstable-msc2867")]
impl std::ops::Deref for UnstableMarkedUnreadEventContent {
    type Target = MarkedUnreadEventContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "unstable-msc2867")]
impl From<MarkedUnreadEventContent> for UnstableMarkedUnreadEventContent {
    fn from(value: MarkedUnreadEventContent) -> Self {
        Self(value)
    }
}

#[cfg(feature = "unstable-msc2867")]
impl From<UnstableMarkedUnreadEventContent> for MarkedUnreadEventContent {
    fn from(value: UnstableMarkedUnreadEventContent) -> Self {
        value.0
    }
}

#[cfg(all(test, feature = "unstable-msc2867"))]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{MarkedUnreadEventContent, UnstableMarkedUnreadEventContent};
    use crate::{AnyRoomAccountDataEvent, RoomAccountDataEvent};

    #[test]
    fn deserialize() {
        let raw_unstable_marked_unread = json!({
            "type": "com.famedly.marked_unread",
            "content": {
                "unread": true,
            },
        });
        let unstable_marked_unread_account_data =
            from_json_value::<AnyRoomAccountDataEvent>(raw_unstable_marked_unread).unwrap();
        assert_matches!(
            unstable_marked_unread_account_data,
            AnyRoomAccountDataEvent::UnstableMarkedUnread(unstable_marked_unread)
        );
        assert!(unstable_marked_unread.content.unread);

        let raw_marked_unread = json!({
            "type": "m.marked_unread",
            "content": {
                "unread": true,
            },
        });
        let marked_unread_account_data =
            from_json_value::<AnyRoomAccountDataEvent>(raw_marked_unread).unwrap();
        assert_matches!(
            marked_unread_account_data,
            AnyRoomAccountDataEvent::MarkedUnread(marked_unread)
        );
        assert!(marked_unread.content.unread);
    }

    #[test]
    fn serialize() {
        let marked_unread = MarkedUnreadEventContent::new(true);
        let marked_unread_account_data = RoomAccountDataEvent { content: marked_unread.clone() };
        assert_eq!(
            to_json_value(marked_unread_account_data).unwrap(),
            json!({
                "type": "m.marked_unread",
                "content": {
                    "unread": true,
                },
            })
        );

        let unstable_marked_unread = UnstableMarkedUnreadEventContent::from(marked_unread);
        let unstable_marked_unread_account_data =
            RoomAccountDataEvent { content: unstable_marked_unread };
        assert_eq!(
            to_json_value(unstable_marked_unread_account_data).unwrap(),
            json!({
                "type": "com.famedly.marked_unread",
                "content": {
                    "unread": true,
                },
            })
        );
    }
}
