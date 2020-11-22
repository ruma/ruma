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
#[ruma_event(type = "m.room.avatar")]
pub struct AvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// Information about the avatar thumbnail image.
    /// URL of the avatar image.
    #[cfg(not(feature = "unstable-synapse-quirks"))]
    pub url: String,
    /// Information about the avatar thumbnail image.
    /// URL of the avatar image.
    #[cfg(feature = "unstable-synapse-quirks")]
    pub url: Option<String>,
}

impl AvatarEventContent {
    /// Create an `AvatarEventContent` from the given image URL.
    pub fn new(url: String) -> Self {
        #[cfg(feature = "unstable-synapse-quirks")]
        let url = Some(url);

        Self { info: None, url }
    }
}
