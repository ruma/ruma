//! Types for the *m.typing* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::{RoomId, UserId};

ruma_event! {
    /// Informs the client of the list of users currently typing.
    TypingEvent {
        kind: Event,
        event_type: Typing,
        fields: {
            /// The unique identifier for the room associated with this event.
            pub room_id: RoomId,
        },
        content: {
            /// The list of user IDs typing in this room, if any.
            pub user_ids: Vec<UserId>,
        },
    }
}
