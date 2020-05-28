//! Endpoint to send join events to remote homeservers.

pub mod v1;

use ruma_events::EventJson;
use serde::{Deserialize, Serialize};

use crate::pdu::Pdu;

/// Full state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomState {
    /// The resident server's DNS name.
    pub origin: String,
    /// The full set of authorization events that make up the state of the room,
    /// and their authorization events, recursively.
    pub auth_chain: Vec<EventJson<Pdu>>,
    /// The room state.
    pub state: Vec<EventJson<Pdu>>,
}
