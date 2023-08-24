//! Types for the `m.poll.response` event.

use std::{ops::Deref, vec};

use ruma_common::OwnedEventId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{start::PollContentBlock, validate_selections, PollResponseData};
use crate::relation::Reference;

/// The payload for a poll response event.
///
/// This is the event content that should be sent for room versions that support extensible events.
/// As of Matrix 1.7, none of the stable room versions (1 through 10) support extensible events.
///
/// To send a poll response event for a room version that does not support extensible events, use
/// [`UnstablePollResponseEventContent`].
///
/// [`UnstablePollResponseEventContent`]: super::unstable_response::UnstablePollResponseEventContent
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.poll.response", kind = MessageLike)]
pub struct PollResponseEventContent {
    /// The user's selection.
    #[serde(rename = "m.selections")]
    pub selections: SelectionsContentBlock,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl PollResponseEventContent {
    /// Creates a new `PollResponseEventContent` that responds to the given poll start event ID,
    /// with the given poll response content.
    pub fn new(selections: SelectionsContentBlock, poll_start_id: OwnedEventId) -> Self {
        Self {
            selections,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: Reference::new(poll_start_id),
        }
    }
}

impl OriginalSyncPollResponseEvent {
    /// Get the data from this response necessary to compile poll results.
    pub fn data(&self) -> PollResponseData<'_> {
        PollResponseData {
            sender: &self.sender,
            origin_server_ts: self.origin_server_ts,
            selections: &self.content.selections,
        }
    }
}

impl OriginalPollResponseEvent {
    /// Get the data from this response necessary to compile poll results.
    pub fn data(&self) -> PollResponseData<'_> {
        PollResponseData {
            sender: &self.sender,
            origin_server_ts: self.origin_server_ts,
            selections: &self.content.selections,
        }
    }
}

/// A block for selections content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SelectionsContentBlock(Vec<String>);

impl SelectionsContentBlock {
    /// Whether this `SelectionsContentBlock` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Validate these selections against the given `PollContentBlock`.
    ///
    /// Returns the list of valid selections in this `SelectionsContentBlock`, or `None` if there is
    /// no valid selection.
    pub fn validate<'a>(
        &'a self,
        poll: &PollContentBlock,
    ) -> Option<impl Iterator<Item = &'a str>> {
        let answer_ids = poll.answers.iter().map(|a| a.id.as_str()).collect();
        validate_selections(&answer_ids, poll.max_selections, &self.0)
    }
}

impl From<Vec<String>> for SelectionsContentBlock {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl IntoIterator for SelectionsContentBlock {
    type Item = String;
    type IntoIter = vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<String> for SelectionsContentBlock {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl Deref for SelectionsContentBlock {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
