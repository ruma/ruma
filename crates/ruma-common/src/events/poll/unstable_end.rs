//! Types for the `org.matrix.msc3381.poll.end` event, the unstable version of `m.poll.end`.

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{events::relation::Reference, OwnedEventId};

/// The payload for an unstable poll end event.
///
/// This type can be generated from the unstable poll start and poll response events with
/// [`OriginalSyncUnstablePollStartEvent::compile_results()`].
///
/// This is the event content that should be sent for room versions that don't support extensible
/// events. As of Matrix 1.7, none of the stable room versions (1 through 10) support extensible
/// events.
///
/// To send a poll end event for a room version that supports extensible events, use
/// [`PollEndEventContent`].
///
/// [`OriginalSyncUnstablePollStartEvent::compile_results()`]: super::unstable_start::OriginalSyncUnstablePollStartEvent::compile_results
/// [`PollEndEventContent`]: super::end::PollEndEventContent
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.end", kind = MessageLike)]
pub struct UnstablePollEndEventContent {
    /// The text representation of the results.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: String,

    /// The poll end content.
    #[serde(default, rename = "org.matrix.msc3381.poll.end")]
    pub poll_end: UnstablePollEndContentBlock,

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl UnstablePollEndEventContent {
    /// Creates a new `PollEndEventContent` with the given fallback representation and
    /// that responds to the given poll start event ID.
    pub fn new(text: impl Into<String>, poll_start_id: OwnedEventId) -> Self {
        Self {
            text: text.into(),
            poll_end: UnstablePollEndContentBlock {},
            relates_to: Reference::new(poll_start_id),
        }
    }
}

/// A block for the results of a poll.
///
/// This is currently an empty struct.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstablePollEndContentBlock {}
