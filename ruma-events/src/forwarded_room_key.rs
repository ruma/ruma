//! Types for the *m.forwarded_room_key* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::{EventEncryptionAlgorithm, RoomId};
use serde::{Deserialize, Serialize};

/// The payload for `ForwardedRoomKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.forwarded_room_key")]
pub struct ForwardedRoomKeyToDeviceEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: RoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,

    /// The Ed25519 key of the device which initiated the session originally.
    ///
    /// It is "claimed" because the receiving device has no way to tell that the original
    /// room_key actually came from a device which owns the private part of this key unless
    /// they have done device verification.
    pub sender_claimed_ed25519_key: String,

    /// Chain of Curve25519 keys.
    ///
    /// It starts out empty, but each time the key is forwarded to another device, the
    /// previous sender in the chain is added to the end of the list. For example, if the
    /// key is forwarded from A to B to C, this field is empty between A and B, and contains
    /// A's Curve25519 key between B and C.
    pub forwarding_curve25519_key_chain: Vec<String>,
}
