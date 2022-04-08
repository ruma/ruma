//! Types for the [`m.room.message.feedback`] event.
//!
//! [`m.room.message.feedback`]: https://spec.matrix.org/v1.2/client-server-api/#mroommessagefeedback

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, OwnedEventId, PrivOwnedStr};

/// The content of an `m.room.message.feedback` event.
///
/// An acknowledgement of a message.
///
/// N.B.: Usage of this event is discouraged in favor of the receipts module. Most clients will
/// not recognize this event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message.feedback", kind = MessageLike)]
pub struct RoomMessageFeedbackEventContent {
    /// The event that this feedback is related to.
    pub target_event_id: OwnedEventId,

    /// The type of feedback.
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,
}

impl RoomMessageFeedbackEventContent {
    /// Create a `RoomMessageFeedbackEventContent` from the given target event id and feedback type.
    pub fn new(target_event_id: OwnedEventId, feedback_type: FeedbackType) -> Self {
        Self { target_event_id, feedback_type }
    }
}

/// A type of feedback.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FeedbackType {
    /// Sent when a message is received.
    Delivered,

    /// Sent when a message has been observed by the end user.
    Read,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl FeedbackType {
    /// Creates a string slice from this `FeedbackType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
