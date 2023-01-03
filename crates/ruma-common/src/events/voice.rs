//! Types for voice message events ([MSC3245]).
//!
//! [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    audio::AudioContent, file::FileContent, message::TextContentBlock, room::message::Relation,
};

/// The payload for an extensible voice message.
///
/// This is the new primary type introduced in [MSC3245] and can be sent in rooms with a version
/// that doesn't support extensible events. See the documentation of the [`message`] module for more
/// information.
///
/// [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.voice", kind = MessageLike, without_relation)]
pub struct VoiceEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

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
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::events::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<VoiceEventContentWithoutRelation>>,
}

impl VoiceEventContent {
    /// Creates a new `VoiceEventContent` with the given fallback representation and file.
    pub fn new(text: TextContentBlock, file: FileContent) -> Self {
        Self { text, file, audio: Default::default(), voice: Default::default(), relates_to: None }
    }

    /// Creates a new `VoiceEventContent` with the given plain text fallback representation and
    /// file.
    pub fn plain(text: impl Into<String>, file: FileContent) -> Self {
        Self {
            text: TextContentBlock::plain(text),
            file,
            audio: Default::default(),
            voice: Default::default(),
            relates_to: None,
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
