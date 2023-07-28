//! Types for the `org.matrix.msc3381.poll.response` event, the unstable version of
//! `m.poll.response`.

use std::ops::Deref;

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{events::relation::Reference, OwnedEventId};

use super::unstable_start::UnstablePollStartContentBlock;

/// The payload for an unstable poll response event.
///
/// This is the event content that should be sent for room versions that don't support extensible
/// events. As of Matrix 1.7, none of the stable room versions (1 through 10) support extensible
/// events.
///
/// To send a poll response event for a room version that supports extensible events, use
/// [`PollResponseEventContent`].
///
/// [`PollResponseEventContent`]: super::response::PollResponseEventContent
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.response", kind = MessageLike)]
pub struct UnstablePollResponseEventContent {
    /// The response's content.
    #[serde(rename = "org.matrix.msc3381.poll.response")]
    pub poll_response: UnstablePollResponseContentBlock,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl UnstablePollResponseEventContent {
    /// Creates a new `UnstablePollResponseEventContent` that responds to the given poll start event
    /// ID, with the given answers.
    pub fn new(answers: Vec<String>, poll_start_id: OwnedEventId) -> Self {
        Self {
            poll_response: UnstablePollResponseContentBlock::new(answers),
            relates_to: Reference::new(poll_start_id),
        }
    }
}

/// An unstable block for poll response content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstablePollResponseContentBlock {
    /// The selected answers for the response.
    pub answers: Vec<String>,
}

impl UnstablePollResponseContentBlock {
    /// Creates a new `UnstablePollResponseContentBlock` with the given answers.
    pub fn new(answers: Vec<String>) -> Self {
        Self { answers }
    }

    /// Validate these selections against the given `UnstablePollStartContentBlock`.
    ///
    /// Returns the list of valid selections in this `UnstablePollResponseContentBlock`, or `None`
    /// if there is no valid selection.
    pub fn validate(
        &self,
        poll: &UnstablePollStartContentBlock,
    ) -> Option<impl Iterator<Item = &str>> {
        // Vote is spoiled if any answer is unknown.
        if self.answers.iter().any(|s| !poll.answers.iter().any(|a| a.id == *s)) {
            return None;
        }

        // Fallback to the maximum value for usize because we can't have more selections than that
        // in memory.
        let max_selections: usize = poll.max_selections.try_into().unwrap_or(usize::MAX);

        Some(self.answers.iter().take(max_selections).map(Deref::deref))
    }
}

impl From<Vec<String>> for UnstablePollResponseContentBlock {
    fn from(value: Vec<String>) -> Self {
        Self::new(value)
    }
}
