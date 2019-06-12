//! Types for the *m.room.message.feedback* event.

use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

room_event! {
    /// An acknowledgement of a message.
    ///
    /// N.B.: Usage of this event is discouraged in favor of the receipts module. Most clients will
    /// not recognise this event.
    pub struct FeedbackEvent(FeedbackEventContent) {}
}

/// The payload of an *m.room.message.feedback* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FeedbackEventContent {
    /// The event that this feedback is related to.
    pub target_event_id: EventId,
    /// The type of feedback.
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,
}

/// A type of feedback.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum FeedbackType {
    /// Sent when a message is received.
    #[serde(rename = "delivered")]
    Delivered,

    /// Sent when a message has been observed by the end user.
    #[serde(rename = "read")]
    Read,
}

impl_enum! {
    FeedbackType {
        Delivered => "delivered",
        Read => "read",
    }
}
