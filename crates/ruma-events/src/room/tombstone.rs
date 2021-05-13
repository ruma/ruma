//! Types for the *m.room.tombstone* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// A state event signifying that a room has been upgraded to a different room version, and that
/// clients should go there.
pub type TombstoneEvent = StateEvent<TombstoneEventContent>;

/// The payload for `TombstoneEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.tombstone", kind = State)]
pub struct TombstoneEventContent {
    /// A server-defined message.
    ///
    /// If you activate the `compat` feature, this field being absent in JSON will give you an
    /// empty string here.
    #[cfg_attr(feature = "compat", serde(default))]
    pub body: String,

    /// The new room the client should be visiting.
    pub replacement_room: RoomId,
}
