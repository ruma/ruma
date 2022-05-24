//! Types for the [`m.poll.end`] event.

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::ReferenceRelation;
use crate::OwnedEventId;

/// The payload for a poll end event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.end", alias = "m.poll.end", kind = MessageLike)]
pub struct PollEndEventContent {
    /// The poll end content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.end", alias = "m.poll.end")]
    pub poll_end: PollEndContent,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: ReferenceRelation,
}

impl PollEndEventContent {
    /// Creates a new `PollEndEventContent` that responds to the given poll start event ID,
    /// with the given poll end content.
    pub fn new(poll_end: PollEndContent, poll_start_id: OwnedEventId) -> Self {
        Self { poll_end, relates_to: ReferenceRelation::new(poll_start_id) }
    }
}

/// Poll end content.
///
/// This is currently empty.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollEndContent {}

impl PollEndContent {
    /// Creates a new empty `PollEndContent`.
    pub fn new() -> Self {
        Self {}
    }
}
