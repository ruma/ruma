//! Types for the *m.room_key* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::RoomId;

use super::Algorithm;

ruma_event! {
    /// This event type is used to exchange keys for end-to-end encryption.
    ///
    /// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
    RoomKeyEvent {
        kind: Event,
        event_type: "m.room_key",
        content: {
            /// The encryption algorithm the key in this event is to be used with.
            ///
            /// Must be `m.megolm.v1.aes-sha2`.
            pub algorithm: Algorithm,

            /// The room where the key is used.
            pub room_id: RoomId,

            /// The ID of the session that the key is for.
            pub session_id: String,

            /// The key to be exchanged.
            pub session_key: String,
        }
    }
}
