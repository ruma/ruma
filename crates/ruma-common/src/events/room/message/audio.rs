use std::time::Duration;

use js_int::UInt;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3245")]
use crate::events::voice::VoiceContent;
#[cfg(feature = "unstable-msc3246")]
use crate::events::{
    audio::AudioContent,
    file::{FileContent, FileContentInfo},
    message::MessageContent,
};
use crate::{
    events::room::{EncryptedFile, MediaSource},
    OwnedMxcUri,
};

/// The payload for an audio message.
///
/// With the `unstable-msc3246` feature, this type contains the transitional format of
/// [`AudioEventContent`] and with the `unstable-msc3245` feature, this type also contains the
/// transitional format of [`VoiceEventContent`]. See the documentation of the [`message`] module
/// for more information.
///
/// [`AudioEventContent`]: crate::events::audio::AudioEventContent
/// [`VoiceEventContent`]: crate::events::voice::VoiceEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.audio")]
#[cfg_attr(
    feature = "unstable-msc3246",
    serde(from = "super::content_serde::AudioMessageEventContentDeHelper")
)]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

    /// The source of the audio clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata for the audio clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<AudioInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event audio info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(rename = "org.matrix.msc1767.audio", skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioContent>,

    /// Extensible-event voice flag of the message.
    ///
    /// If present, this should be represented as a voice message.
    #[cfg(feature = "unstable-msc3245")]
    #[serde(rename = "org.matrix.msc3245.voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceContent>,
}

impl AudioMessageEventContent {
    /// Creates a new non-encrypted `AudioMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<AudioInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3246")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3246")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            #[cfg(feature = "unstable-msc3246")]
            audio: Some(info.as_deref().map_or_else(AudioContent::default, Into::into)),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `AudioMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3246")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3246")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3246")]
            audio: Some(AudioContent::default()),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `AudioMessageEventContent` with the given message, file info and audio info.
    #[cfg(feature = "unstable-msc3246")]
    pub fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        audio: AudioContent,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = AudioInfo::from_extensible_content(file.info.as_deref(), &audio).map(Box::new);

        Self {
            message: Some(message),
            file: Some(file),
            audio: Some(audio),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source,
            info,
        }
    }

    /// Create a new `AudioMessageEventContent` with the given message, file info, audio info and
    /// voice flag.
    #[cfg(feature = "unstable-msc3245")]
    pub fn from_extensible_voice_content(
        message: MessageContent,
        file: FileContent,
        audio: AudioContent,
        voice: VoiceContent,
    ) -> Self {
        let mut content = Self::from_extensible_content(message, file, audio);
        content.voice = Some(voice);
        content
    }
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(
        with = "crate::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The mimetype of the audio, e.g. "audio/aac".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the audio clip in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

impl AudioInfo {
    /// Creates an empty `AudioInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an `AudioInfo` from the given file info and audio info.
    #[cfg(feature = "unstable-msc3246")]
    pub fn from_extensible_content(
        file_info: Option<&FileContentInfo>,
        audio: &AudioContent,
    ) -> Option<Self> {
        if file_info.is_none() && audio.is_empty() {
            None
        } else {
            let (mimetype, size) = file_info
                .map(|info| (info.mimetype.to_owned(), info.size.to_owned()))
                .unwrap_or_default();
            let AudioContent { duration, .. } = audio;

            Some(Self { duration: duration.to_owned(), mimetype, size })
        }
    }
}
