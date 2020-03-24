//! Types for the *m.room.encryption* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;

use crate::Algorithm;

ruma_event! {
    /// Defines how messages sent in this room should be encrypted.
    EncryptionEvent {
        kind: StateEvent,
        event_type: "m.room.encryption",
        content: {
            /// The encryption algorithm to be used to encrypt messages sent in this room.
            ///
            /// Must be `m.megolm.v1.aes-sha2`.
            pub algorithm: Algorithm,

            /// How long the session should be used before changing it.
            ///
            /// 604800000 (a week) is the recommended default.
            pub rotation_period_ms: Option<UInt>,

            /// How many messages should be sent before changing the session.
            ///
            /// 100 is the recommended default.
            pub rotation_period_msgs: Option<UInt>,
        },
    }
}
