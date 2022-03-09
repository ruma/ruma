//! Types for the [`m.room.pinned_events`] event.
//!
//! [`m.room.pinned_events`]: https://spec.matrix.org/v1.2/client-server-api/#mroompinned_events

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EventId;

/// The content of an `m.room.pinned_events` event.
///
/// Used to "pin" particular events in a room for other participants to review later.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.pinned_events", kind = State)]
pub struct RoomPinnedEventsEventContent {
    /// An ordered list of event IDs to pin.
    pub pinned: Vec<Box<EventId>>,
}

impl RoomPinnedEventsEventContent {
    /// Creates a new `RoomPinnedEventsEventContent` with the given events.
    pub fn new(pinned: Vec<Box<EventId>>) -> Self {
        Self { pinned }
    }
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use std::convert::TryInto;

    use crate::{server_name, EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId};

    use super::RoomPinnedEventsEventContent;
    use crate::events::{StateEvent, Unsigned};

    #[test]
    fn serialization_deserialization() {
        let mut content: RoomPinnedEventsEventContent =
            RoomPinnedEventsEventContent { pinned: Vec::new() };
        let server_name = server_name!("example.com");

        content.pinned.push(EventId::new(server_name));
        content.pinned.push(EventId::new(server_name));

        let event = StateEvent {
            content: content.clone(),
            event_id: EventId::new(server_name),
            origin_server_ts: MilliSecondsSinceUnixEpoch(1_432_804_485_886_u64.try_into().unwrap()),
            prev_content: None,
            room_id: RoomId::new(server_name),
            sender: UserId::new(server_name),
            state_key: "".into(),
            unsigned: Unsigned::default(),
        };

        let serialized_event = serde_json::to_string(&event).unwrap();
        let parsed_event: StateEvent<RoomPinnedEventsEventContent> =
            serde_json::from_str(&serialized_event).unwrap();

        assert_eq!(parsed_event.event_id, event.event_id);
        assert_eq!(parsed_event.room_id, event.room_id);
        assert_eq!(parsed_event.sender, event.sender);
        assert_eq!(parsed_event.state_key, event.state_key);
        assert_eq!(parsed_event.origin_server_ts, event.origin_server_ts);

        assert_eq!(parsed_event.content.pinned, event.content.pinned);
        assert_eq!(parsed_event.content.pinned[0], content.pinned[0]);
        assert_eq!(parsed_event.content.pinned[1], content.pinned[1]);
    }
}
