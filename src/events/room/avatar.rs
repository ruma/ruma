//! Types for the *m.room.avatar* event.

use events::EventType;
use super::ImageInfo;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
#[derive(Debug, Deserialize, Serialize)]
pub struct AvatarEvent {
    pub content: AvatarEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<AvatarEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of an `AvatarEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AvatarEventContent {
    pub info: ImageInfo,
    pub thumbnail_info: ImageInfo,
    pub thumbnail_url: String,
    pub url: String,
}
