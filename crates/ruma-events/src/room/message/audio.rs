use std::time::Duration;

use js_int::UInt;
use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::room::{EncryptedFile, MediaSource};

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.audio")]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    ///
    /// If the `filename` field is not set or has the same value, this is the filename of the
    /// uploaded file. Otherwise, this should be interpreted as a user-written media caption.
    pub body: String,

    /// Formatted form of the message `body`.
    ///
    /// This should only be set if the body represents a caption.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The source of the audio clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata for the audio clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<AudioInfo>>,

    /// Extensible event fallback data for audio messages, from the
    /// [first version of MSC3245][msc].
    ///
    /// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/83f6c5b469c1d78f714e335dcaa25354b255ffa5/proposals/3245-voice-messages.md
    #[cfg(feature = "unstable-msc3245-v1-compat")]
    #[serde(rename = "org.matrix.msc1767.audio", skip_serializing_if = "Option::is_none")]
    pub audio: Option<UnstableAudioDetailsContentBlock>,

    /// Extensible event fallback data for voice messages, from the
    /// [first version of MSC3245][msc].
    ///
    /// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/83f6c5b469c1d78f714e335dcaa25354b255ffa5/proposals/3245-voice-messages.md
    #[cfg(feature = "unstable-msc3245-v1-compat")]
    #[serde(rename = "org.matrix.msc3245.voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<UnstableVoiceContentBlock>,
}

impl AudioMessageEventContent {
    /// Creates a new `AudioMessageEventContent` with the given body and source.
    pub fn new(body: String, source: MediaSource) -> Self {
        Self {
            body,
            formatted: None,
            filename: None,
            source,
            info: None,
            #[cfg(feature = "unstable-msc3245-v1-compat")]
            audio: None,
            #[cfg(feature = "unstable-msc3245-v1-compat")]
            voice: None,
        }
    }

    /// Creates a new non-encrypted `AudioMessageEventContent` with the given body and url.
    pub fn plain(body: String, url: OwnedMxcUri) -> Self {
        Self::new(body, MediaSource::Plain(url))
    }

    /// Creates a new encrypted `AudioMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self::new(body, MediaSource::Encrypted(Box::new(file)))
    }

    /// Creates a new `AudioMessageEventContent` from `self` with the `info` field set to the given
    /// value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn info(self, info: impl Into<Option<Box<AudioInfo>>>) -> Self {
        Self { info: info.into(), ..self }
    }
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(
        with = "ruma_common::serde::duration::opt_ms",
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
}

/// Extensible event fallback data for audio messages, from the
/// [first version of MSC3245][msc].
///
/// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/83f6c5b469c1d78f714e335dcaa25354b255ffa5/proposals/3245-voice-messages.md
#[cfg(feature = "unstable-msc3245-v1-compat")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstableAudioDetailsContentBlock {
    /// The duration of the audio in milliseconds.
    ///
    /// Note that the MSC says this should be in seconds but for compatibility with the Element
    /// clients, this uses milliseconds.
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub duration: Duration,

    /// The waveform representation of the audio content, if any.
    ///
    /// This is optional and defaults to an empty array.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waveform: Vec<UnstableAmplitude>,
}

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl UnstableAudioDetailsContentBlock {
    /// Creates a new `UnstableAudioDetailsContentBlock ` with the given duration and waveform.
    pub fn new(duration: Duration, waveform: Vec<UnstableAmplitude>) -> Self {
        Self { duration, waveform }
    }
}

/// Extensible event fallback data for voice messages, from the
/// [first version of MSC3245][msc].
///
/// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/83f6c5b469c1d78f714e335dcaa25354b255ffa5/proposals/3245-voice-messages.md
#[cfg(feature = "unstable-msc3245-v1-compat")]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstableVoiceContentBlock {}

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl UnstableVoiceContentBlock {
    /// Creates a new `UnstableVoiceContentBlock`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The unstable version of the amplitude of a waveform sample.
///
/// Must be an integer between 0 and 1024.
#[cfg(feature = "unstable-msc3245-v1-compat")]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UnstableAmplitude(UInt);

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl UnstableAmplitude {
    /// The smallest value that can be represented by this type, 0.
    pub const MIN: u16 = 0;

    /// The largest value that can be represented by this type, 1024.
    pub const MAX: u16 = 1024;

    /// Creates a new `UnstableAmplitude` with the given value.
    ///
    /// It will saturate if it is bigger than [`UnstableAmplitude::MAX`].
    pub fn new(value: u16) -> Self {
        Self(value.min(Self::MAX).into())
    }

    /// The value of this `UnstableAmplitude`.
    pub fn get(&self) -> UInt {
        self.0
    }
}

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl From<u16> for UnstableAmplitude {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl<'de> Deserialize<'de> for UnstableAmplitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uint = UInt::deserialize(deserializer)?;
        Ok(Self(uint.min(Self::MAX.into())))
    }
}
