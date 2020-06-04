//! Types for the *m.room.topic* event.

use ruma_events_macros::{FromRaw, StateEventContent};
use serde::Serialize;

/// A topic is a short message detailing what is currently being discussed in the room.
#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[ruma_event(type = "m.room.topic")]
pub struct TopicEventContent {
    /// The topic text.
    pub topic: String,
}
