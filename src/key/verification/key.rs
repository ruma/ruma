//! Types for the *m.key.verification.key* event.

use serde::{Deserialize, Serialize};

event! {
    /// Sends the ephemeral public key for a device to the partner device.
    ///
    /// Typically sent as a to-device event.
    pub struct KeyEvent(KeyEventContent) {}
}

/// The payload of an *m.key.verification.key* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct KeyEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// The device's ephemeral public key, encoded as unpadded Base64.
    pub key: String,
}
