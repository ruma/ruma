//! Types for the [`m.room.policy`] event.
//!
//! [`m.room.policy`]: https://spec.matrix.org/v1.18/client-server-api/#mroompolicy

use std::collections::BTreeMap;

use ruma_common::{OwnedServerName, SigningKeyAlgorithm, serde::Base64};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an [`m.room.policy`] event.
///
/// A [Policy Server] configuration.
///
/// If invalid or not set, the room does not use a Policy Server.
///
/// [`m.room.policy`]: https://spec.matrix.org/v1.18/client-server-api/#mroompolicy
/// [Policy Server]: https://spec.matrix.org/v1.18/client-server-api/#policy-servers
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.policy", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomPolicyEventContent {
    /// The server name to use as a Policy Server.
    ///
    /// MUST have a joined user in the room.
    pub via: OwnedServerName,

    /// The public keys for the Policy Server.
    ///
    /// MUST contain at least `ed25519`.
    pub public_keys: BTreeMap<SigningKeyAlgorithm, Base64>,
}

impl RoomPolicyEventContent {
    /// The signing key ID that must be used by the Policy Server for the ed25519 signature.
    pub const POLICY_SERVER_ED25519_SIGNING_KEY_ID: &str = "ed25519:policy_server";

    /// Creates a new `RoomPolicyEventContent` with the given server name and ed25519 public key.
    pub fn new(via: OwnedServerName, ed25519_public_key: Base64) -> Self {
        Self { via, public_keys: [(SigningKeyAlgorithm::Ed25519, ed25519_public_key)].into() }
    }
}
