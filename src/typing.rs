//! Types for the *m.typing* event.

use ruma_identifiers::{RoomId, UserId};
use serde::{Deserialize, Serialize};

event! {
    /// Informs the client of the list of users currently typing.
    pub struct TypingEvent(TypingEventContent) {
        /// The unique identifier for the room associated with this event.
        pub room_id: RoomId
    }
}

/// The payload of a `TypingEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<UserId>,
}
