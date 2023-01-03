//! Types for extensible video message events ([MSC3553]).
//!
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553

use std::time::Duration;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::FileContent, image::ThumbnailContent, message::TextContentBlock, room::message::Relation,
};

/// The payload for an extensible video message.
///
/// This is the new primary type introduced in [MSC3553] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.video", kind = MessageLike, without_relation)]
pub struct VideoEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

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
    #[serde(rename = "m.caption", default, skip_serializing_if = "TextContentBlock::is_empty")]
    pub caption: TextContentBlock,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::events::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<VideoEventContentWithoutRelation>>,
}

impl VideoEventContent {
    /// Creates a new `VideoEventContent` with the given fallback representation and file.
    pub fn new(text: TextContentBlock, file: FileContent) -> Self {
        Self {
            text,
            file,
            video: Default::default(),
            thumbnail: Default::default(),
            caption: Default::default(),
            relates_to: None,
        }
    }

    /// Creates a new `VideoEventContent` with the given plain text fallback representation and
    /// file.
    pub fn plain(text: impl Into<String>, file: FileContent) -> Self {
        Self {
            text: TextContentBlock::plain(text),
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

    /// Whether this `VideoContent` is empty.
    pub fn is_empty(&self) -> bool {
        self.height.is_none() && self.width.is_none() && self.duration.is_none()
    }
}
