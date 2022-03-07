//! `PUT /_matrix/federation/*/send_join/{roomId}/{eventId}`
//!
//! Endpoint to send join events to remote homeservers.

pub mod v1;
pub mod v2;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

/// Full state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomState {
    #[cfg(not(feature = "unstable-pre-spec"))]
    /// The resident server's DNS name.
    pub origin: String,

    /// The full set of authorization events that make up the state of the room,
    /// and their authorization events, recursively.
    pub auth_chain: Vec<Box<RawJsonValue>>,

    /// The room state.
    pub state: Vec<Box<RawJsonValue>>,
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
    ///
    /// With the `unstable-pre-spec` feature, this method doesn't take any parameters.
    /// See [matrix-spec#374](https://github.com/matrix-org/matrix-spec/issues/374).
    pub fn new(origin: String) -> Self {
        Self { origin, auth_chain: Vec::new(), state: Vec::new() }
    }

    #[cfg(feature = "unstable-pre-spec")]
    /// Creates an empty `RoomState` with the given `origin`.
    ///
    /// Without the `unstable-pre-spec` feature, this method takes a parameter for the origin
    /// See [matrix-spec#374](https://github.com/matrix-org/matrix-spec/issues/374).
    pub fn new() -> Self {
        Self { auth_chain: Vec::new(), state: Vec::new() }
    }
}
