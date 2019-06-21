//! Types for the *m.key.verification.request* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;
use ruma_identifiers::DeviceId;

use super::VerificationMethod;

ruma_event! {
    /// Requests a key verification with another user's devices.
    ///
    /// Typically sent as a to-device event.
    RequestEvent {
        kind: Event,
        event_type: KeyVerificationRequest,
        content: {
            /// The device ID which is initiating the request.
            pub from_device: DeviceId,

            /// An opaque identifier for the verification request.
            ///
            /// Must be unique with respect to the devices involved.
            pub transaction_id: String,

            /// The verification methods supported by the sender.
            pub methods: Vec<VerificationMethod>,

            /// The POSIX timestamp in milliseconds for when the request was made.
            ///
            /// If the request is in the future by more than 5 minutes or more than 10 minutes in
            /// the past, the message should be ignored by the receiver.
            pub timestamp: UInt,
        },
    }
}
