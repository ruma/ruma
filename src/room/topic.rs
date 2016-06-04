//! Types for the *m.room.topic* event.

use core::EventType;

/// A topic is a short message detailing what is currently being discussed in the room.
pub struct TopicEvent {
    content: TopicEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<TopicEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `TopicEvent`.
pub struct TopicEventContent {
    /// The topic text.
    topic: String,
}
