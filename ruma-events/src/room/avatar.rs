//! Types for the *m.room.avatar* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use super::ImageInfo;
use crate::StateEvent;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
pub type AvatarEvent = StateEvent<AvatarEventContent>;

/// The payload for `AvatarEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[cfg_attr(feature = "unstable-pre-spec", derive(Default))]
#[ruma_event(type = "m.room.avatar")]
pub struct AvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// URL of the avatar image.
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub url: String,

    /// URL of the avatar image.
    #[cfg(feature = "unstable-pre-spec")]
    pub url: Option<String>,
}

impl AvatarEventContent {
    /// Create an `AvatarEventContent` from the given image URL.
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub fn new(url: String) -> Self {
        Self { info: None, url }
    }

    /// Create an empty `AvatarEventContent`.
    #[cfg(feature = "unstable-pre-spec")]
    pub fn new() -> Self {
        Self::default()
    }
}
