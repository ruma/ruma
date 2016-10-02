//! Types for the *m.room.topic* event.

state_event! {
    /// A topic is a short message detailing what is currently being discussed in the room.
    pub struct TopicEvent(TopicEventContent) {}
}

/// The payload of a `TopicEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TopicEventContent {
    /// The topic text.
    pub topic: String,
}
