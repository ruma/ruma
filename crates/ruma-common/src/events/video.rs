//! Types for extensible video message events ([MSC3553]).
//!
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553

use std::time::Duration;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::FileContent, image::ThumbnailContent, message::MessageContent, room::message::Relation,
};

/// The payload for an extensible video message.
///
/// This is the new primary type introduced in [MSC3553] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.video", kind = MessageLike, without_relation)]
pub struct VideoEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The file content of the message.
    #[serde(rename = "m.file")]
    pub file: FileContent,

    /// The video content of the message.
    #[serde(rename = "m.video")]
    pub video: Box<VideoContent>,

    /// The thumbnails of the message.
    #[serde(rename = "m.thumbnail", default, skip_serializing_if = "Vec::is_empty")]
    pub thumbnail: Vec<ThumbnailContent>,

    /// The captions of the message.
    #[serde(
        rename = "m.caption",
        with = "super::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl VideoEventContent {
    /// Creates a new `VideoEventContent` with the given plain text message and file.
    pub fn plain(message: impl Into<String>, file: FileContent) -> Self {
        Self {
            message: MessageContent::plain(message),
            file,
            video: Default::default(),
            thumbnail: Default::default(),
            caption: Default::default(),
            relates_to: None,
        }
    }

    /// Creates a new `VideoEventContent` with the given message and file.
    pub fn with_message(message: MessageContent, file: FileContent) -> Self {
        Self {
            message,
            file,
            video: Default::default(),
            thumbnail: Default::default(),
            caption: Default::default(),
            relates_to: None,
        }
    }
}

/// Video content.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoContent {
    /// The height of the video in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the video in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The duration of the video in milliseconds.
    #[serde(
        with = "crate::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,
}

impl VideoContent {
    /// Creates a new empty `VideoContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `VideoContent` with the given optional height, width and duration.
    pub(crate) fn from_room_message_content(
        height: Option<UInt>,
        width: Option<UInt>,
        duration: Option<Duration>,
    ) -> Self {
        Self { height, width, duration }
    }

    /// Whether this `VideoContent` is empty.
    pub fn is_empty(&self) -> bool {
        self.height.is_none() && self.width.is_none() && self.duration.is_none()
    }
}
