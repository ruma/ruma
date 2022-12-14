//! Types for the [`m.room.pinned_events`] event.
//!
//! [`m.room.pinned_events`]: https://spec.matrix.org/v1.4/client-server-api/#mroompinned_events

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{events::EmptyStateKey, OwnedEventId};

/// The content of an `m.room.pinned_events` event.
///
/// Used to "pin" particular events in a room for other participants to review later.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.pinned_events", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomPinnedEventsEventContent {
    /// An ordered list of event IDs to pin.
    pub pinned: Vec<OwnedEventId>,
}

impl RoomPinnedEventsEventContent {
    /// Creates a new `RoomPinnedEventsEventContent` with the given events.
    pub fn new(pinned: Vec<OwnedEventId>) -> Self {
        Self { pinned }
    }
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use super::RoomPinnedEventsEventContent;
    use crate::{server_name, EventId};

    #[test]
    fn serialization_deserialization() {
        let mut content: RoomPinnedEventsEventContent =
            RoomPinnedEventsEventContent { pinned: Vec::new() };
        let server_name = server_name!("example.com");

        content.pinned.push(EventId::new(server_name));
        content.pinned.push(EventId::new(server_name));

        let serialized = serde_json::to_string(&content).unwrap();
        let parsed_content: RoomPinnedEventsEventContent =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed_content.pinned, content.pinned);
    }
}
