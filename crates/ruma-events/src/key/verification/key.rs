//! Types for the [`m.key.verification.key`] event.
//!
//! [`m.key.verification.key`]: https://spec.matrix.org/v1.1/client-server-api/#mkeyverificationkey

use ruma_events_macros::EventContent;
use ruma_identifiers::TransactionId;
use ruma_serde::Base64;
use serde::{Deserialize, Serialize};

use super::Relation;

/// The content of a to-device `m.key.verification.key` event.
///
/// Sends the ephemeral public key for a device to the partner device.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.key", kind = ToDevice)]
pub struct ToDeviceKeyVerificationKeyEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the `m.key.verification.start` message.
    pub transaction_id: Box<TransactionId>,

    /// The device's ephemeral public key, encoded as unpadded base64.
    pub key: Base64,
}

impl ToDeviceKeyVerificationKeyEventContent {
    /// Creates a new `ToDeviceKeyVerificationKeyEventContent` with the given transaction ID and
    /// key.
    pub fn new(transaction_id: Box<TransactionId>, key: Base64) -> Self {
        Self { transaction_id, key }
    }
}

/// The content of an in-room `m.key.verification.key` event.
///
/// Sends the ephemeral public key for a device to the partner device.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.key", kind = Message)]
pub struct KeyVerificationKeyEventContent {
    /// The device's ephemeral public key, encoded as unpadded base64.
    pub key: Base64,

    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

impl KeyVerificationKeyEventContent {
    /// Creates a new `KeyVerificationKeyEventContent` with the given key and relation.
    pub fn new(key: Base64, relates_to: Relation) -> Self {
        Self { key, relates_to }
    }
}
