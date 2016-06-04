//! Types for the *m.room.avatar* event.

use core::EventType;
use super::ImageInfo;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
pub struct AvatarEvent {
    content: AvatarEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<AvatarEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of an `AvatarEvent`.
pub struct AvatarEventContent {
    info: ImageInfo,
    thumbnail_info: ImageInfo,
    thumbnail_url: String,
    url: String,
}
