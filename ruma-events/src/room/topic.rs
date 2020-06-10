//! Types for the *m.room.topic* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// A topic is a short message detailing what is currently being discussed in the room.
pub type TopicEvent = StateEvent<TopicEventContent>;

/// The payload for `TopicEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.topic")]
pub struct TopicEventContent {
    /// The topic text.
    pub topic: String,
}
