//! Types for the *m.room.topic* event.

use events::EventType;

/// A topic is a short message detailing what is currently being discussed in the room.
#[derive(Debug, Deserialize, Serialize)]
pub struct TopicEvent {
    pub content: TopicEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<TopicEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `TopicEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TopicEventContent {
    /// The topic text.
    pub topic: String,
}
