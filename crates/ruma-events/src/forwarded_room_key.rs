//! Types for the [`m.forwarded_room_key`] event.
//!
//! [`m.forwarded_room_key`]: https://spec.matrix.org/latest/client-server-api/#mforwarded_room_key

use ruma_common::{EventEncryptionAlgorithm, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.forwarded_room_key` event.
///
/// To create an instance of this type, first create a `ToDeviceForwardedRoomKeyEventContentInit`
/// and convert it via `ToDeviceForwardedRoomKeyEventContent::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.forwarded_room_key", kind = ToDevice)]
pub struct ToDeviceForwardedRoomKeyEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

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

    /// Used to mark key if allowed for shared history.
    ///
    /// Defaults to `false`.
    #[cfg(feature = "unstable-msc3061")]
    #[serde(
        default,
        rename = "org.matrix.msc3061.shared_history",
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    pub shared_history: bool,
}

/// Initial set of fields of `ToDeviceForwardedRoomKeyEventContent`.
///
/// This struct will not be updated even if additional fields are added to `ConditionalPushRule` in
/// a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ToDeviceForwardedRoomKeyEventContentInit {
    /// The encryption algorithm the key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

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

impl From<ToDeviceForwardedRoomKeyEventContentInit> for ToDeviceForwardedRoomKeyEventContent {
    fn from(init: ToDeviceForwardedRoomKeyEventContentInit) -> Self {
        Self {
            algorithm: init.algorithm,
            room_id: init.room_id,
            sender_key: init.sender_key,
            session_id: init.session_id,
            session_key: init.session_key,
            sender_claimed_ed25519_key: init.sender_claimed_ed25519_key,
            forwarding_curve25519_key_chain: init.forwarding_curve25519_key_chain,
            #[cfg(feature = "unstable-msc3061")]
            shared_history: false,
        }
    }
}
