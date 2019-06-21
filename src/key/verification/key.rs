//! Types for the *m.key.verification.key* event.

use ruma_events_macros::ruma_event;

ruma_event! {
    /// Sends the ephemeral public key for a device to the partner device.
    ///
    /// Typically sent as a to-device event.
    KeyEvent {
        kind: Event,
        event_type: KeyVerificationKey,
        content: {
            /// An opaque identifier for the verification process.
            ///
            /// Must be the same as the one used for the *m.key.verification.start* message.
            pub transaction_id: String,

            /// The device's ephemeral public key, encoded as unpadded Base64.
            pub key: String,
        },
    }
}
