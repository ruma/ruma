//! Types for the [`m.key.verification.mac`] event.
//!
//! [`m.key.verification.mac`]: https://spec.matrix.org/v1.1/client-server-api/#mkeyverificationmac

use std::collections::BTreeMap;

use ruma_events_macros::EventContent;
use ruma_serde::Base64;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use super::Relation;

/// The content of a to-device `m.key.verification.` event.
///
/// Sends the MAC of a device's key to the partner device.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.mac", kind = ToDevice)]
pub struct ToDeviceKeyVerificationMacEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the `m.key.verification.start` message.
    pub transaction_id: String,

    /// A map of the key ID to the MAC of the key, using the algorithm in the verification process.
    ///
    /// The MAC is encoded as unpadded base64.
    pub mac: BTreeMap<String, Base64>,

    /// The MAC of the comma-separated, sorted, list of key IDs given in the `mac` property,
    /// encoded as unpadded base64.
    pub keys: Base64,
}

impl ToDeviceKeyVerificationMacEventContent {
    /// Creates a new `ToDeviceKeyVerificationMacEventContent` with the given transaction ID, key ID
    /// to MAC map and key MAC.
    pub fn new(transaction_id: String, mac: BTreeMap<String, Base64>, keys: Base64) -> Self {
        Self { transaction_id, mac, keys }
    }
}

/// The content of an in-room `m.key.verification.` event.
///
/// Sends the MAC of a device's key to the partner device.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.mac", kind = Message)]
pub struct KeyVerificationMacEventContent {
    /// A map of the key ID to the MAC of the key, using the algorithm in the verification process.
    ///
    /// The MAC is encoded as unpadded base64.
    pub mac: BTreeMap<String, Base64>,

    /// The MAC of the comma-separated, sorted, list of key IDs given in the `mac` property,
    /// encoded as unpadded base64.
    pub keys: Base64,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

#[cfg(feature = "unstable-pre-spec")]
impl KeyVerificationMacEventContent {
    /// Creates a new `KeyVerificationMacEventContent` with the given key ID to MAC map, key MAC and
    /// relation.
    pub fn new(mac: BTreeMap<String, Base64>, keys: Base64, relates_to: Relation) -> Self {
        Self { mac, keys, relates_to }
    }
}
