//! Types for the *m.room.topic* event.

use events::StateEvent;

/// A topic is a short message detailing what is currently being discussed in the room.
pub type TopicEvent = StateEvent<TopicEventContent>;

/// The payload of a `TopicEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TopicEventContent {
    /// The topic text.
    pub topic: String,
}
