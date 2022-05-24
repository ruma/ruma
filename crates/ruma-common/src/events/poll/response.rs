//! Types for the [`m.poll.response`] event.

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::ReferenceRelation;
use crate::OwnedEventId;

/// The payload for a poll response event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.response", alias = "m.poll.response", kind = MessageLike)]
pub struct PollResponseEventContent {
    /// The poll response content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.response", alias = "m.poll.response")]
    pub poll_response: PollResponseContent,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: ReferenceRelation,
}

impl PollResponseEventContent {
    /// Creates a new `PollResponseEventContent` that responds to the given poll start event ID,
    /// with the given poll response content.
    pub fn new(poll_response: PollResponseContent, poll_start_id: OwnedEventId) -> Self {
        Self { poll_response, relates_to: ReferenceRelation::new(poll_start_id) }
    }
}

/// Poll response content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollResponseContent {
    /// The IDs of the selected answers of the poll.
    ///
    /// It should be truncated to `max_selections` from the related poll start event.
    ///  
    /// If this is an empty array or includes unknown IDs, this vote should be considered as
    /// spoiled.
    pub answers: Vec<String>,
}

impl PollResponseContent {
    /// Creates a new `PollResponseContent` with the given answers.
    pub fn new(answers: Vec<String>) -> Self {
        Self { answers }
    }
}
