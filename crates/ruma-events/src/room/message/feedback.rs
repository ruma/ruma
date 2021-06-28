//! Types for the *m.room.message.feedback* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::EventId;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::MessageEvent;

/// An acknowledgement of a message.
///
/// N.B.: Usage of this event is discouraged in favor of the receipts module. Most clients will
/// not recognize this event.
pub type FeedbackEvent = MessageEvent<FeedbackEventContent>;

/// The payload for `FeedbackEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message.feedback", kind = Message)]
pub struct FeedbackEventContent {
    /// The event that this feedback is related to.
    pub target_event_id: EventId,

    /// The type of feedback.
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,
}

impl FeedbackEventContent {
    /// Create a `FeedbackEventContent` from the given target event id and feedback type.
    pub fn new(target_event_id: EventId, feedback_type: FeedbackType) -> Self {
        Self { target_event_id, feedback_type }
    }
}

/// A type of feedback.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FeedbackType {
    /// Sent when a message is received.
    Delivered,

    /// Sent when a message has been observed by the end user.
    Read,

    #[doc(hidden)]
    _Custom(String),
}

impl FeedbackType {
    /// Creates a string slice from this `FeedbackType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
