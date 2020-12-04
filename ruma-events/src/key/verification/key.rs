//! Types for the *m.key.verification.key* event.

use ruma_events_macros::BasicEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;
#[cfg(feature = "unstable-pre-spec")]
use crate::MessageEvent;

/// Sends the ephemeral public key for a device to the partner device.
#[cfg(feature = "unstable-pre-spec")]
pub type KeyEvent = MessageEvent<KeyEventContent>;

/// The payload for a to-device `KeyEvent`.
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

/// The payload for in-room `KeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.key")]
#[cfg(feature = "unstable-pre-spec")]
pub struct KeyEventContent {
    /// The device's ephemeral public key, encoded as unpadded Base64.
    pub key: String,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}
