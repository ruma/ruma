//! Types for the [`m.room.encryption`] event.
//!
//! [`m.room.encryption`]: https://spec.matrix.org/latest/client-server-api/#mroomencryption

use js_int::{uint, UInt};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, EventEncryptionAlgorithm};

/// The content of an `m.room.encryption` event.
///
/// Defines how messages sent in this room should be encrypted.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.encryption", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomEncryptionEventContent {
    /// The encryption algorithm to be used to encrypt messages sent in this room.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: EventEncryptionAlgorithm,

    /// How long the session should be used before changing it.
    ///
    /// `uint!(604800000)` (a week) is the recommended default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_period_ms: Option<UInt>,

    /// How many messages should be sent before changing the session.
    ///
    /// `uint!(100)` is the recommended default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_period_msgs: Option<UInt>,
}

impl RoomEncryptionEventContent {
    /// Creates a new `RoomEncryptionEventContent` with the given algorithm.
    pub fn new(algorithm: EventEncryptionAlgorithm) -> Self {
        Self { algorithm, rotation_period_ms: None, rotation_period_msgs: None }
    }

    /// Creates a new `RoomEncryptionEventContent` with the mandatory algorithm and the recommended
    /// defaults.
    ///
    /// Note that changing the values of the fields is not a breaking change and you shouldn't rely
    /// on those specific values.
    pub fn with_recommended_defaults() -> Self {
        // Defaults defined at <https://spec.matrix.org/latest/client-server-api/#mroomencryption>
        Self {
            algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
            rotation_period_ms: Some(uint!(604_800_000)),
            rotation_period_msgs: Some(uint!(100)),
        }
    }
}
