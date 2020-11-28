//! Endpoint to send join events to remote homeservers.

pub mod v1;
pub mod v2;

use ruma_events::pdu::Pdu;
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

/// Full state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomState {
    #[cfg(not(feature = "unstable-pre-spec"))]
    /// The resident server's DNS name.
    pub origin: String,

    /// The full set of authorization events that make up the state of the room,
    /// and their authorization events, recursively.
    pub auth_chain: Vec<Raw<Pdu>>,

    /// The room state.
    pub state: Vec<Raw<Pdu>>,
}

#[cfg(feature = "unstable-pre-spec")]
impl Default for RoomState {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomState {
    #[cfg(not(feature = "unstable-pre-spec"))]
    /// Creates an empty `RoomState` with the given `origin`.
    pub fn new(origin: String) -> Self {
        Self { origin, auth_chain: Vec::new(), state: Vec::new() }
    }

    #[cfg(feature = "unstable-pre-spec")]
    /// Creates an empty `RoomState` with the given `origin`.
    pub fn new() -> Self {
        Self { auth_chain: Vec::new(), state: Vec::new() }
    }
}
