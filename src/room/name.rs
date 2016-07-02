//! Types for the *m.room.name* event.

use StateEvent;

/// A human-friendly room name designed to be displayed to the end-user.
pub type NameEvent = StateEvent<NameEventContent, ()>;

/// The payload of a `NameEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub name: String,
}
