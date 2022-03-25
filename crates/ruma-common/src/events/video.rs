//! Types for extensible video message events ([MSC3553]).
//!
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553

use std::time::Duration;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::FileContent,
    image::{Captions, ThumbnailContent},
    message::MessageContent,
    room::message::Relation,
};

/// The payload for an extensible video message.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.video", kind = MessageLike)]
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
    #[serde(rename = "m.caption", default, skip_serializing_if = "Captions::is_empty")]
    pub caption: Captions,

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
        with = "ruma_common::serde::duration::opt_ms",
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
}
