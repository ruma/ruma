//! Types for the *m.room.avatar* event.

use events::StateEvent;
use super::ImageInfo;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
pub type AvatarEvent = StateEvent<AvatarEventContent>;

/// The payload of an `AvatarEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AvatarEventContent {
    pub info: ImageInfo,
    pub thumbnail_info: ImageInfo,
    pub thumbnail_url: String,
    pub url: String,
}
