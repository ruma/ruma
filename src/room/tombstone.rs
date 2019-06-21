//! Types for the *m.room.tombstone* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::RoomId;

ruma_event! {
    /// A state event signifying that a room has been upgraded to a different room version, and that
    /// clients should go there.
    TombstoneEvent {
        kind: StateEvent,
        event_type: RoomTombstone,
        content: {
            /// A server-defined message.
            pub body: String,

            /// The new room the client should be visiting.
            pub replacement_room: RoomId,
        },
    }
}
