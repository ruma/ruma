//! Types for the *m.room.pinned_events* event.

use ruma_events_macros::StateEventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// Used to "pin" particular events in a room for other participants to review later.
pub type PinnedEventsEvent = StateEvent<PinnedEventsEventContent>;

/// The payload for `PinnedEventsEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.pinned_events")]
pub struct PinnedEventsEventContent {
    /// An ordered list of event IDs to pin.
    pub pinned: Vec<EventId>,
}

impl PinnedEventsEventContent {
    /// Creates a new `PinnedEventsEventContent` with the given events.
    pub fn new(pinned: Vec<EventId>) -> Self {
        Self { pinned }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use ruma_identifiers::{EventId, RoomId, ServerName, UserId};
    use ruma_serde::Raw;
    use serde_json::to_string;

    use super::PinnedEventsEventContent;
    use crate::{StateEvent, Unsigned};

    #[test]
    fn serialization_deserialization() {
        let mut content: PinnedEventsEventContent = PinnedEventsEventContent { pinned: Vec::new() };
        let server_name = <&ServerName>::try_from("example.com").unwrap();

        content.pinned.push(EventId::new(server_name));
        content.pinned.push(EventId::new(server_name));

        let event = StateEvent {
            content: content.clone(),
            event_id: EventId::new(server_name),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1_432_804_485_886u64),
            prev_content: None,
            room_id: RoomId::new(server_name),
            sender: UserId::new(server_name),
            state_key: "".into(),
            unsigned: Unsigned::default(),
        };

        let serialized_event = to_string(&event).unwrap();
        let parsed_event =
            serde_json::from_str::<Raw<StateEvent<PinnedEventsEventContent>>>(&serialized_event)
                .unwrap()
                .deserialize()
                .unwrap();

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
