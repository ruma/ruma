//! Types for the *m.key.verification.mac* event.

use ruma_signatures::SignatureSet;
use serde::{Deserialize, Serialize};

event! {
    /// Sends the MAC of a device's key to the partner device.
    ///
    /// Typically sent as a to-device event.
    pub struct MacEvent(MacEventContent) {}
}

/// The payload for an *m.key.verification.mac* event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MacEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// A map of the key ID to the MAC of the key, using the algorithm in the verification process.
    ///
    /// The MAC is encoded as unpadded Base64.
    pub mac: SignatureSet,

    /// The MAC of the comma-separated, sorted, list of key IDs given in the `mac` property, encoded
    /// as unpadded Base64.
    pub keys: String,
}
