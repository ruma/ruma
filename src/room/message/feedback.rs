//! Types for the *m.room.message.feedback* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

ruma_event! {
    /// An acknowledgement of a message.
    ///
    /// N.B.: Usage of this event is discouraged in favor of the receipts module. Most clients will
    /// not recognize this event.
    FeedbackEvent {
        kind: RoomEvent,
        event_type: "m.room.message.feedback",
        content: {
            /// The event that this feedback is related to.
            pub target_event_id: EventId,

            /// The type of feedback.
            #[serde(rename = "type")]
            pub feedback_type: FeedbackType,
        },
    }
}

/// A type of feedback.
#[derive(Clone, Copy, Debug, Display, EnumString, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FeedbackType {
    /// Sent when a message is received.
    Delivered,

    /// Sent when a message has been observed by the end user.
    Read,
}
