//! Types for the `org.matrix.msc3381.poll.response` event, the unstable version of
//! `m.poll.response`.

use ruma_common::OwnedEventId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{unstable_start::UnstablePollStartContentBlock, validate_selections, PollResponseData};
use crate::relation::Reference;

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

impl OriginalSyncUnstablePollResponseEvent {
    /// Get the data from this response necessary to compile poll results.
    pub fn data(&self) -> PollResponseData<'_> {
        PollResponseData {
            sender: &self.sender,
            origin_server_ts: self.origin_server_ts,
            selections: &self.content.poll_response.answers,
        }
    }
}

impl OriginalUnstablePollResponseEvent {
    /// Get the data from this response necessary to compile poll results.
    pub fn data(&self) -> PollResponseData<'_> {
        PollResponseData {
            sender: &self.sender,
            origin_server_ts: self.origin_server_ts,
            selections: &self.content.poll_response.answers,
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
    pub fn validate<'a>(
        &'a self,
        poll: &UnstablePollStartContentBlock,
    ) -> Option<impl Iterator<Item = &'a str>> {
        let answer_ids = poll.answers.iter().map(|a| a.id.as_str()).collect();
        validate_selections(&answer_ids, poll.max_selections, &self.answers)
    }
}

impl From<Vec<String>> for UnstablePollResponseContentBlock {
    fn from(value: Vec<String>) -> Self {
        Self::new(value)
    }
}
