//! Types for the *m.room.pinned_events* event.

use ruma_events_macros::{FromRaw, StateEventContent};
use ruma_identifiers::EventId;
use serde::Serialize;

/// Used to "pin" particular events in a room for other participants to review later.
#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[ruma_event(type = "m.room.pinned_events")]
pub struct PinnedEventsEventContent {
    /// An ordered list of event IDs to pin.
    pub pinned: Vec<EventId>,
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::to_string;

    use super::PinnedEventsEventContent;
    use crate::{EventJson, StateEvent, UnsignedData};

    #[test]
    fn serialization_deserialization() {
        let mut content: PinnedEventsEventContent = PinnedEventsEventContent { pinned: Vec::new() };

        content.pinned.push(EventId::new("example.com").unwrap());
        content.pinned.push(EventId::new("example.com").unwrap());

        let event = StateEvent {
            content: content.clone(),
            event_id: EventId::new("example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1_432_804_485_886u64),
            prev_content: None,
            room_id: RoomId::new("example.com").unwrap(),
            sender: UserId::new("example.com").unwrap(),
            state_key: "".to_string(),
            unsigned: UnsignedData::default(),
        };

        let serialized_event = to_string(&event).unwrap();
        let parsed_event = serde_json::from_str::<EventJson<StateEvent<PinnedEventsEventContent>>>(
            &serialized_event,
        )
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
