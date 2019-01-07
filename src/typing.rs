//! Types for the *m.typing* event.

use ruma_identifiers::{RoomId, UserId};

event! {
    /// Informs the client of the list of users currently typing.
    pub struct TypingEvent(TypingEventContent) {
        /// The unique identifier for the room associated with this event.
        ///
        /// This can be `None` if the event came from a context where there is
        /// no ambiguity which room it belongs to, like a `/sync` response for example.
        #[serde(skip_serializing_if="Option::is_none")]
        pub room_id: Option<RoomId>
    }
}

/// The payload of a `TypingEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<UserId>,
}
