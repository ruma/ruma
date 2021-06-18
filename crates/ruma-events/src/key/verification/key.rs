//! Types for the *m.key.verification.key* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Sends the ephemeral public key for a device to the partner device.
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub type KeyEvent = MessageEvent<KeyEventContent>;

/// The payload for a to-device `KeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.key", kind = ToDevice)]
pub struct KeyToDeviceEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// The device's ephemeral public key, encoded as unpadded Base64.
    pub key: String,
}

impl KeyToDeviceEventContent {
    /// Creates a new `KeyToDeviceEventContent` with the given transaction ID and key.
    pub fn new(transaction_id: String, key: String) -> Self {
        Self { transaction_id, key }
    }
}

/// The payload for in-room `KeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.key", kind = Message)]
pub struct KeyEventContent {
    /// The device's ephemeral public key, encoded as unpadded Base64.
    pub key: String,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

#[cfg(feature = "unstable-pre-spec")]
impl KeyEventContent {
    /// Creates a new `KeyEventContent` with the given key and relation.
    pub fn new(key: String, relation: Relation) -> Self {
        Self { key, relation }
    }
}
