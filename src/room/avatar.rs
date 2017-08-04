//! Types for the *m.room.avatar* event.

use super::ImageInfo;

state_event! {
    /// A picture that is associated with the room.
    ///
    /// This can be displayed alongside the room information.
    pub struct AvatarEvent(AvatarEventContent) {}
}

/// The payload of an `AvatarEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if="Option::is_none")]
    pub info: Option<ImageInfo>,
    /// Information about the avatar thumbnail image.
    #[serde(skip_serializing_if="Option::is_none")]
    pub thumbnail_info: Option<ImageInfo>,
    /// URL of the avatar thumbnail image.
    #[serde(skip_serializing_if="Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// URL of the avatar image.
    pub url: String,
}
