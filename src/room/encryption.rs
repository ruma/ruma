//! Types for the *m.room.encryption* event.

use js_int::UInt;
use ruma_events_macros::{FromRaw, StateEventContent};
use serde::Serialize;

use crate::Algorithm;

/// Defines how messages sent in this room should be encrypted.
#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[ruma_event(type = "m.room.encryption")]
pub struct EncryptionEventContent {
    /// The encryption algorithm to be used to encrypt messages sent in this room.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: Algorithm,

    /// How long the session should be used before changing it.
    ///
    /// 604800000 (a week) is the recommended default.
    pub rotation_period_ms: Option<UInt>,

    /// How many messages should be sent before changing the session.
    ///
    /// 100 is the recommended default.
    pub rotation_period_msgs: Option<UInt>,
}
