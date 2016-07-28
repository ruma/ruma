//! Types for the *m.room.create* event.

use ruma_identifiers::UserId;

use StateEvent;

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
pub type CreateEvent = StateEvent<CreateEventContent, ()>;

/// The payload of a `CreateEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventContent {
    /// The `user_id` of the room creator. This is set by the homeserver.
    pub creator: UserId,
    /// Whether or not this room's data should be transferred to other homeservers.
    pub federate: bool,
}
