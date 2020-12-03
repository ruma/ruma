//! Types for the *m.key.verification.key* event.

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

/// The payload for `KeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.key")]
pub struct KeyToDeviceEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// The device's ephemeral public key, encoded as unpadded Base64.
    pub key: String,
}
