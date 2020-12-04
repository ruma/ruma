//! Types for the *m.key.verification.mac* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Sends the MAC of a device's key to the partner device.
#[cfg(feature = "unstable-pre-spec")]
pub type MacEvent = MessageEvent<MacEventContent>;

/// The payload for a to-device `MacEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.mac")]
pub struct MacToDeviceEventContent {
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

/// The payload for an in-room `MacEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.mac")]
#[cfg(feature = "unstable-pre-spec")]
pub struct MacEventContent {
    /// A map of the key ID to the MAC of the key, using the algorithm in the verification process.
    ///
    /// The MAC is encoded as unpadded Base64.
    pub mac: BTreeMap<String, String>,

    /// The MAC of the comma-separated, sorted, list of key IDs given in the `mac` property,
    /// encoded as unpadded Base64.
    pub keys: String,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}
