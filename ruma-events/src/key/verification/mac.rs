//! Types for the *m.key.verification.mac* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Sends the MAC of a device's key to the partner device.
///
/// Typically sent as a to-device event.
pub type MacEvent = BasicEvent<MacEventContent>;

/// The payload for `MacEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.mac")]
pub struct MacEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// A map of the key ID to the MAC of the key, using the algorithm in the verification process.
    ///
    /// The MAC is encoded as unpadded Base64.
    pub mac: BTreeMap<String, String>,

    /// The MAC of the comma-separated, sorted, list of key IDs given in the `mac` property,
    /// encoded as unpadded Base64.
    pub keys: String,
}
