//! Types for extensible video message events ([MSC3553]).
//!
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553

use std::time::Duration;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::FileContent,
    image::ThumbnailContent,
    message::MessageContent,
    room::message::{
        MessageType, Relation, RoomMessageEventContent, VideoInfo, VideoMessageEventContent,
    },
};

/// The payload for an extensible video message.
///
/// This is the new primary type introduced in [MSC3553] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `VideoEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Video`]. You can convert it back with
/// [`VideoEventContent::from_video_room_message()`].
///
/// [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Video`]: super::room::message::MessageType::Video
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

    /// Create a new `VideoEventContent` from the given `VideoMessageEventContent` and optional
    /// relation.
    pub fn from_video_room_message(
        content: VideoMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let VideoMessageEventContent {
            body,
            source,
            info,
            message,
            file,
            video,
            thumbnail,
            caption,
        } = content;
        let VideoInfo {
            duration,
            height,
            width,
            mimetype,
            size,
            thumbnail_info,
            thumbnail_source,
            ..
        } = info.map(|info| *info).unwrap_or_default();

        let message = message.unwrap_or_else(|| MessageContent::plain(body));
        let file = file.unwrap_or_else(|| {
            FileContent::from_room_message_content(source, None, mimetype, size)
        });
        let video = video.unwrap_or_else(|| {
            Box::new(VideoContent::from_room_message_content(height, width, duration))
        });
        let thumbnail = thumbnail.unwrap_or_else(|| {
            ThumbnailContent::from_room_message_content(thumbnail_source, thumbnail_info)
                .into_iter()
                .collect()
        });

        Self { message, file, video, thumbnail, caption, relates_to }
    }
}

impl From<VideoEventContent> for RoomMessageEventContent {
    fn from(content: VideoEventContent) -> Self {
        let VideoEventContent { message, file, video, thumbnail, caption, relates_to } = content;

        Self {
            msgtype: MessageType::Video(VideoMessageEventContent::from_extensible_content(
                message, file, video, thumbnail, caption,
            )),
            relates_to,
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
