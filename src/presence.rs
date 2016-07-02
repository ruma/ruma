//! Types for the *m.presence* event.

use Event;

/// Informs the client of a user's presence state change.
pub type PresenceEvent = Event<PresenceEventContent, PresenceEventExtraContent>;

/// The payload of a `PresenceEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PresenceEventContent {
    /// The current avatar URL for this user.
    pub avatar_url: Option<String>,

    /// Whether or not the user is currently active.
    pub currently_active: bool,

    /// The current display name for this user.
    pub displayname: Option<String>,

    /// The last time since this used performed some action, in milliseconds.
    pub last_active_ago: Option<u64>,

    /// The presence state for this user.
    pub presence: PresenceState,

    /// The unique identifier for the user associated with this event.
    pub user_id: String,
}

/// A description of a user's connectivity and availability for chat.
#[derive(Debug, Deserialize, Serialize)]
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

/// Extra content for a `PresenceEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PresenceEventExtraContent {
    /// The unique identifier for the event.
    pub event_id: String,
}
