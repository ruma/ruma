//! Types for the *m.room.create* event.

use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use serde::{Deserialize, Serialize};

state_event! {
    /// This is the first event in a room and cannot be changed. It acts as the root of all other
    /// events.
    pub struct CreateEvent(CreateEventContent) {}
}

/// The payload of a `CreateEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateEventContent {
    /// The `user_id` of the room creator. This is set by the homeserver.
    pub creator: UserId,
    /// Whether or not this room's data should be transferred to other homeservers.
    #[serde(rename = "m.federate")]
    pub federate: Option<bool>,
    /// The version of the room. Defaults to "1" if the key does not exist.
    pub room_version: RoomVersionId,
    /// A reference to the room this room replaces, if the previous room was upgraded.
    pub predecessor: PreviousRoom,
}

/// A reference to an old room replaced during a room version upgrade.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PreviousRoom {
    /// The ID of the old room.
    pub room_id: RoomId,
    /// The event ID of the last known event in the old room.
    pub event_id: EventId,
}
