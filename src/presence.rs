//! Types for the *m.presence* event.

use core::EventType;

/// Informs the client of a user's presence state change.
pub struct PresenceEvent {
    content: PresenceEventContent,
    event_id: String,
    event_type: EventType,
}

/// The payload of a `PresenceEvent`.
pub struct PresenceEventContent {
    /// The current avatar URL for this user.
    avatar_url: Option<String>,
    /// The current display name for this user.
    displayname: Option<String>,
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
