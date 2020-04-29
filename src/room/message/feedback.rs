//! Types for the *m.room.message.feedback* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum FeedbackType {
    /// Sent when a message is received.
    #[serde(rename = "delivered")]
    Delivered,

    /// Sent when a message has been observed by the end user.
    #[serde(rename = "read")]
    Read,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    FeedbackType {
        Delivered => "delivered",
        Read => "read",
    }
}
