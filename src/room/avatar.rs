//! Types for the *m.room.avatar* event.

use serde::{Deserialize, Serialize};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,
    /// Information about the avatar thumbnail image.
    /// URL of the avatar image.
    pub url: String,
}
