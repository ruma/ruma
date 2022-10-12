//! Types for voice message events ([MSC3245]).
//!
//! [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    audio::AudioContent,
    file::FileContent,
    message::{MessageContent, TryFromExtensibleError},
    room::message::{
        AudioInfo, AudioMessageEventContent, MessageType, Relation, RoomMessageEventContent,
    },
};

/// The payload for an extensible voice message.
///
/// This is the new primary type introduced in [MSC3245] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `VoiceEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Audio`] with the `m.voice` flag. You can convert it back with
/// [`VoiceEventContent::try_from_audio_room_message()`].
///
/// [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Audio`]: super::room::message::MessageType::Audio
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.voice", kind = MessageLike, without_relation)]
pub struct VoiceEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The file content of the message.
    #[serde(rename = "m.file")]
    pub file: FileContent,

    /// The audio content of the message.
    #[serde(rename = "m.audio")]
    pub audio: AudioContent,

    /// The voice content of the message.
    #[serde(rename = "m.voice")]
    pub voice: VoiceContent,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl VoiceEventContent {
    /// Creates a new `VoiceEventContent` with the given plain text representation and file.
    pub fn plain(message: impl Into<String>, file: FileContent) -> Self {
        Self {
            message: MessageContent::plain(message),
            file,
            audio: Default::default(),
            voice: Default::default(),
            relates_to: None,
        }
    }

    /// Creates a new `VoiceEventContent` with the given message and file.
    pub fn with_message(message: MessageContent, file: FileContent) -> Self {
        Self {
            message,
            file,
            audio: Default::default(),
            voice: Default::default(),
            relates_to: None,
        }
    }

    /// Create a new `VoiceEventContent` from the given `AudioMessageEventContent` and optional
    /// relation.
    ///
    /// This can fail if the `AudioMessageEventContent` is not a voice message.
    pub fn try_from_audio_room_message(
        content: AudioMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Result<Self, TryFromExtensibleError> {
        let AudioMessageEventContent { body, source, info, message, file, audio, voice } = content;
        let AudioInfo { duration, mimetype, size } = info.map(|info| *info).unwrap_or_default();

        let message = message.unwrap_or_else(|| MessageContent::plain(body));
        let file = file.unwrap_or_else(|| {
            FileContent::from_room_message_content(source, None, mimetype, size)
        });
        let audio = audio
            .or_else(|| duration.map(AudioContent::from_room_message_content))
            .unwrap_or_default();
        let voice = if let Some(voice) = voice {
            voice
        } else {
            return Err(TryFromExtensibleError::MissingField("m.voice".to_owned()));
        };

        Ok(Self { message, file, audio, voice, relates_to })
    }
}

impl From<VoiceEventContent> for RoomMessageEventContent {
    fn from(content: VoiceEventContent) -> Self {
        let VoiceEventContent { message, file, audio, voice, relates_to } = content;

        Self {
            msgtype: MessageType::Audio(AudioMessageEventContent::from_extensible_voice_content(
                message, file, audio, voice,
            )),
            relates_to,
        }
    }
}

/// Voice content.
///
/// This is currently empty and used as a flag to mark an audio event that should be displayed as a
/// voice message.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VoiceContent {}

impl VoiceContent {
    /// Creates a new empty `VoiceContent`.
    pub fn new() -> Self {
        Self::default()
    }
}
