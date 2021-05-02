//! Types for the *m.typing* event.

use ruma_events_macros::EphemeralRoomEventContent;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::EphemeralRoomEvent;

/// Informs the client who is currently typing in a given room.
pub type TypingEvent = EphemeralRoomEvent<TypingEventContent>;

/// The payload for `TypingEvent`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EphemeralRoomEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.typing")]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<UserId>,
}

impl TypingEventContent {
    /// Creates a new `TypingEventContent` with the given user IDs.
    pub fn new(user_ids: Vec<UserId>) -> Self {
        Self { user_ids }
    }
}
