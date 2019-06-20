//! Types for the *m.fully_read* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::{EventId, RoomId};

ruma_event! {
    /// The current location of the user's read marker in a room.
    ///
    /// This event appears in the user's room account data for the room the marker is applicable
    /// for.
    FullyReadEvent {
        kind: Event,
        event_type: FullyRead,
        fields: {
            /// The unique identifier for the room associated with this event.
            pub room_id: RoomId,
        },
        content: {
            /// The event the user's read marker is located at in the room.
            pub event_id: EventId,
        },
    }
}
