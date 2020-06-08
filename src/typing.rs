//! Types for the *m.typing* event.

use ruma_events_macros::EphemeralRoomEventContent;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

/// Informs the client who is currently typing in a given room.
#[derive(Clone, Debug, Deserialize, Serialize, EphemeralRoomEventContent)]
#[ruma_event(type = "m.typing")]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<UserId>,
}
