//! Types for the [`m.poll.end`] event.

use std::{
    collections::{btree_map, BTreeMap},
    ops::Deref,
};

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    events::{message::TextContentBlock, relation::Reference},
    OwnedEventId,
};

/// The payload for a poll end event.
///
/// This type can be generated from the poll start and poll response events with
/// [`OriginalSyncPollStartEvent::compile_results()`].
///
/// [`OriginalSyncPollStartEvent::compile_results()`]: super::start::OriginalSyncPollStartEvent::compile_results
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.v2.poll.end", alias = "m.poll.end", kind = MessageLike)]
pub struct PollEndEventContent {
    /// The text representation of the results.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// The sender's perspective of the results.
    #[serde(
        rename = "org.matrix.msc3381.v2.poll.results",
        skip_serializing_if = "Option::is_none"
    )]
    pub poll_results: Option<PollResultsContentBlock>,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "crate::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl PollEndEventContent {
    /// Creates a new `PollEndEventContent` with the given fallback representation and
    /// that responds to the given poll start event ID.
    pub fn new(text: TextContentBlock, poll_start_id: OwnedEventId) -> Self {
        Self {
            text,
            poll_results: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: Reference::new(poll_start_id),
        }
    }

    /// Creates a new `PollEndEventContent` with the given plain text fallback representation and
    /// that responds to the given poll start event ID.
    pub fn with_plain_text(plain_text: impl Into<String>, poll_start_id: OwnedEventId) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            poll_results: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: Reference::new(poll_start_id),
        }
    }
}

/// A block for the results of a poll.
///
/// This is a map of answer ID to number of votes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollResultsContentBlock(BTreeMap<String, UInt>);

impl PollResultsContentBlock {
    /// Get these results sorted from the highest number of votes to the lowest.
    ///
    /// Returns a list of `(answer ID, number of votes)`.
    pub fn sorted(&self) -> Vec<(&str, UInt)> {
        let mut sorted = self.0.iter().map(|(id, count)| (id.as_str(), *count)).collect::<Vec<_>>();
        sorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        sorted
    }
}

impl From<BTreeMap<String, UInt>> for PollResultsContentBlock {
    fn from(value: BTreeMap<String, UInt>) -> Self {
        Self(value)
    }
}

impl IntoIterator for PollResultsContentBlock {
    type Item = (String, UInt);
    type IntoIter = btree_map::IntoIter<String, UInt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, UInt)> for PollResultsContentBlock {
    fn from_iter<T: IntoIterator<Item = (String, UInt)>>(iter: T) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

impl Deref for PollResultsContentBlock {
    type Target = BTreeMap<String, UInt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
