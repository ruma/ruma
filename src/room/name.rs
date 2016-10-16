//! Types for the *m.room.name* event.

state_event! {
    /// A human-friendly room name designed to be displayed to the end-user.
    pub struct NameEvent(NameEventContent) {}
}

/// The payload of a `NameEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub name: String,
}
