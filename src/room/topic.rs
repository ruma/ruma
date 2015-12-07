//! Types for the *m.room.topic* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// A topic is a short message detailing what is currently being discussed in the room.
pub struct TopicEvent<'a, 'b> {
    content: TopicEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<TopicEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, TopicEventContent<'a>> for TopicEvent<'a, 'b> {
    fn content(&'a self) -> &'a TopicEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomTopic
    }
}

impl<'a, 'b> RoomEvent<'a, TopicEventContent<'a>> for TopicEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, TopicEventContent<'a>> for TopicEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b TopicEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `TopicEvent`.
pub struct TopicEventContent<'a> {
    /// The topic text.
    topic: &'a str,
}
