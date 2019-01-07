//! Types for the *m.room.pinned_events* event.

use ruma_identifiers::EventId;

state_event! {
    /// Used to "pin" particular events in a room for other participants to review later.
    pub struct PinnedEventsEvent(PinnedEventsContent) {}
}

/// The payload of a `NameEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PinnedEventsContent {
    /// An ordered list of event IDs to pin.
    pub pinned: Vec<EventId>,
}

#[cfg(test)]
mod tests {
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::{from_str, to_string};

    use room::pinned_events::{PinnedEventsContent, PinnedEventsEvent};
    use Event;
    use EventType;
    use RoomEvent;
    use StateEvent;

    #[test]
    fn serialization_deserialization() {
        let mut content: PinnedEventsContent = PinnedEventsContent { pinned: Vec::new() };

        content.pinned.push(EventId::new("example.com").unwrap());
        content.pinned.push(EventId::new("example.com").unwrap());

        let event = PinnedEventsEvent {
            content: content.clone(),
            event_id: EventId::new("example.com").unwrap(),
            event_type: EventType::RoomPinnedEvents,
            origin_server_ts: 1432804485886,
            prev_content: None,
            room_id: Some(RoomId::new("example.com").unwrap()),
            sender: UserId::new("example.com").unwrap(),
            state_key: "".to_string(),
            unsigned: None,
        };

        let serialized_event = to_string(&event).unwrap();
        let parsed_event: PinnedEventsEvent = from_str(&serialized_event).unwrap();

        assert_eq!(parsed_event.event_id(), event.event_id());
        assert_eq!(parsed_event.room_id(), event.room_id());
        assert_eq!(parsed_event.sender(), event.sender());
        assert_eq!(parsed_event.unsigned(), event.unsigned());
        assert_eq!(parsed_event.state_key(), event.state_key());
        assert_eq!(parsed_event.origin_server_ts(), event.origin_server_ts());

        assert_eq!(parsed_event.content().pinned, event.content.pinned);
        assert_eq!(parsed_event.content().pinned[0], content.pinned[0]);
        assert_eq!(parsed_event.content().pinned[1], content.pinned[1]);
    }
}
