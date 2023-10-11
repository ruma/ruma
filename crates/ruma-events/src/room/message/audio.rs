use std::time::Duration;

use js_int::UInt;
use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3245-v1-compat")]
use crate::audio::Amplitude;
use crate::room::{EncryptedFile, MediaSource};

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.audio")]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

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
            source,
            info: None,
            #[cfg(feature = "unstable-msc3245-v1-compat")]
            audio: None,
            #[cfg(feature = "unstable-msc3245-v1-compat")]
            voice: None,
        }
    }

    /// Creates a new non-encrypted `AudioMessageEventContent` with the given bod and url.
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
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstableAudioDetailsContentBlock {
    /// The duration of the audio in seconds.
    #[serde(with = "ruma_common::serde::duration::secs")]
    pub duration: Duration,

    /// The waveform representation of the audio content, if any.
    ///
    /// This is optional and defaults to an empty array.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waveform: Vec<Amplitude>,
}

#[cfg(feature = "unstable-msc3245-v1-compat")]
impl UnstableAudioDetailsContentBlock {
    /// Creates a new `UnstableAudioDetailsContentBlock ` with the given duration and waveform.
    pub fn new(duration: Duration, waveform: Vec<Amplitude>) -> Self {
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
