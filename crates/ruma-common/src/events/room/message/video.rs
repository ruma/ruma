use std::time::Duration;

use js_int::UInt;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3553")]
use crate::events::{
    file::{FileContent, FileContentInfo},
    image::ThumbnailContent,
    message::MessageContent,
    video::VideoContent,
};
use crate::{
    events::room::{EncryptedFile, MediaSource, ThumbnailInfo},
    OwnedMxcUri,
};

/// The payload for a video message.
///
/// With the `unstable-msc3553` feature, this type contains the transitional format of
/// [`VideoEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`VideoEventContent`]: crate::events::video::VideoEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.video")]
#[cfg_attr(
    feature = "unstable-msc3553",
    serde(from = "super::content_serde::VideoMessageEventContentDeHelper")
)]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style", or some kind of content description for
    /// accessibility, e.g. "video attachment".
    pub body: String,

    /// The source of the video clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the video clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<VideoInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event video info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.video", skip_serializing_if = "Option::is_none")]
    pub video: Option<Box<VideoContent>>,

    /// Extensible-event thumbnails of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(
        rename = "org.matrix.msc1767.caption",
        with = "crate::events::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,
}

impl VideoMessageEventContent {
    /// Creates a new non-encrypted `VideoMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<VideoInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3553")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3553")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().and_then(|info| {
                    FileContentInfo::from_room_message_content(
                        None,
                        info.mimetype.to_owned(),
                        info.size.to_owned(),
                    )
                    .map(Box::new)
                }),
            )),
            #[cfg(feature = "unstable-msc3553")]
            video: Some(Box::new(
                info.as_deref()
                    .map(|info| {
                        VideoContent::from_room_message_content(
                            info.height,
                            info.width,
                            info.duration,
                        )
                    })
                    .unwrap_or_default(),
            )),
            #[cfg(feature = "unstable-msc3553")]
            thumbnail: info
                .as_deref()
                .and_then(|info| {
                    ThumbnailContent::from_room_message_content(
                        info.thumbnail_source.to_owned(),
                        info.thumbnail_info.to_owned(),
                    )
                })
                .map(|thumbnail| vec![thumbnail]),
            #[cfg(feature = "unstable-msc3553")]
            caption: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `VideoMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3553")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3553")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3553")]
            video: Some(Box::new(VideoContent::default())),
            #[cfg(feature = "unstable-msc3553")]
            thumbnail: None,
            #[cfg(feature = "unstable-msc3553")]
            caption: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `VideoMessageEventContent` with the given message, file info, video info,
    /// thumbnails and captions.
    #[cfg(feature = "unstable-msc3553")]
    pub(crate) fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        video: Box<VideoContent>,
        thumbnail: Vec<ThumbnailContent>,
        caption: Option<MessageContent>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = VideoInfo::from_extensible_content(file.info.as_deref(), &video, &thumbnail)
            .map(Box::new);
        let thumbnail = if thumbnail.is_empty() { None } else { Some(thumbnail) };

        Self {
            message: Some(message),
            file: Some(file),
            video: Some(video),
            thumbnail,
            caption,
            body,
            source,
            info,
        }
    }
}

/// Metadata about a video.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    #[serde(
        with = "crate::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The height of the video in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the video in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The mimetype of the video, e.g. "video/mp4".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the video in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the video clip.
    #[serde(
        flatten,
        with = "crate::events::room::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl VideoInfo {
    /// Creates an empty `VideoInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `VideoInfo` from the given file info, video info and thumbnail.
    #[cfg(feature = "unstable-msc3553")]
    fn from_extensible_content(
        file_info: Option<&FileContentInfo>,
        video: &VideoContent,
        thumbnail: &[ThumbnailContent],
    ) -> Option<Self> {
        if file_info.is_none() && video.is_empty() && thumbnail.is_empty() {
            None
        } else {
            let (mimetype, size) = file_info
                .map(|info| (info.mimetype.to_owned(), info.size.to_owned()))
                .unwrap_or_default();
            let VideoContent { duration, height, width } = video.to_owned();
            let (thumbnail_source, thumbnail_info) = thumbnail
                .get(0)
                .map(|thumbnail| {
                    let source = (&thumbnail.file).into();
                    let info = ThumbnailInfo::from_extensible_content(
                        thumbnail.file.info.as_deref(),
                        thumbnail.image.as_deref(),
                    )
                    .map(Box::new);
                    (Some(source), info)
                })
                .unwrap_or_default();

            Some(Self {
                duration,
                height,
                width,
                mimetype,
                size,
                thumbnail_info,
                thumbnail_source,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
            })
        }
    }
}
