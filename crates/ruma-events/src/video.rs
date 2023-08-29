//! Types for extensible video message events ([MSC3553]).
//!
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553

use std::time::Duration;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::{CaptionContentBlock, FileContentBlock},
    image::ThumbnailContentBlock,
    message::TextContentBlock,
    room::message::Relation,
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
#[ruma_event(type = "org.matrix.msc1767.video", kind = MessageLike, without_relation)]
pub struct VideoEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// The file content of the message.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file: FileContentBlock,

    /// The video details of the message, if any.
    #[serde(rename = "org.matrix.msc1767.video_details", skip_serializing_if = "Option::is_none")]
    pub video_details: Option<VideoDetailsContentBlock>,

    /// The thumbnails of the message, if any.
    ///
    /// This is optional and defaults to an empty array.
    #[serde(
        rename = "org.matrix.msc1767.thumbnail",
        default,
        skip_serializing_if = "ThumbnailContentBlock::is_empty"
    )]
    pub thumbnail: ThumbnailContentBlock,

    /// The caption of the message, if any.
    #[serde(rename = "org.matrix.msc1767.caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<CaptionContentBlock>,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<VideoEventContentWithoutRelation>>,
}

impl VideoEventContent {
    /// Creates a new `VideoEventContent` with the given fallback representation and file.
    pub fn new(text: TextContentBlock, file: FileContentBlock) -> Self {
        Self {
            text,
            file,
            video_details: None,
            thumbnail: Default::default(),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new `VideoEventContent` with the given plain text fallback representation and
    /// file.
    pub fn with_plain_text(plain_text: impl Into<String>, file: FileContentBlock) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            file,
            video_details: None,
            thumbnail: Default::default(),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}

/// A block for details of video content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoDetailsContentBlock {
    /// The width of the video in pixels.
    pub width: UInt,

    /// The height of the video in pixels.
    pub height: UInt,

    /// The duration of the video in seconds.
    #[serde(
        with = "ruma_common::serde::duration::opt_secs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,
}

impl VideoDetailsContentBlock {
    /// Creates a new `VideoDetailsContentBlock` with the given height and width.
    pub fn new(width: UInt, height: UInt) -> Self {
        Self { width, height, duration: None }
    }
}
