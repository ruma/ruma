//! Types for the *m.room.avatar* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use super::ImageInfo;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.avatar")]
pub struct AvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// Information about the avatar thumbnail image.
    /// URL of the avatar image.
    pub url: String,
}
