//! Types for the *m.presence* event.

use core::Event;

/// Informs the client of a user's presence state change.
pub struct PresenceEvent<'a> {
    content: PresenceEventContent<'a>,
    event_id: &'a str,
}

impl<'a> Event<'a, PresenceEventContent<'a>> for PresenceEvent<'a> {
    fn content(&'a self) -> &'a PresenceEventContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.presence"
    }
}

/// The payload of a `PresenceEvent`.
pub struct PresenceEventContent<'a> {
    /// The current avatar URL for this user.
    avatar_url: Option<&'a str>,
    /// The current display name for this user.
    displayname: Option<&'a str>,
    /// The last time since this used performed some action, in milliseconds.
    last_active_ago: Option<u64>,
    /// The presence state for this user.
    presence: PresenceState,
}

/// A description of a user's connectivity and availability for chat.
pub enum PresenceState {
    /// Connected to the service and available for chat.
    FreeForChat,
    /// Connected to the service but not visible to other users.
    Hidden,
    /// Disconnected from the service.
    Offline,
    /// Connected to the service.
    Online,
    /// Connected to the service but not available for chat.
    Unavailable,
}
