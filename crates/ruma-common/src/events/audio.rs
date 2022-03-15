//! Types for extensible audio message events ([MSC3246]).
//!
//! [MSC3246]: https://github.com/matrix-org/matrix-spec-proposals/pull/3246

use std::{convert::TryFrom, time::Duration};

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod amplitude_serde;
mod waveform_serde;

use waveform_serde::WaveformSerDeHelper;

use super::{file::FileContent, message::MessageContent, room::message::Relation};

/// The payload for an extensible audio message.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.audio", kind = MessageLike)]
pub struct AudioEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The file content of the message.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file: FileContent,

    /// The audio content of the message.
    #[serde(rename = "org.matrix.msc1767.audio")]
    pub audio: AudioContent,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl AudioEventContent {
    /// Creates a new `AudioEventContent` with the given plain text message and file.
    pub fn plain(message: impl Into<String>, file: FileContent) -> Self {
        Self {
            message: MessageContent::plain(message),
            file,
            audio: Default::default(),
            relates_to: None,
        }
    }

    /// Creates a new `AudioEventContent` with the given message and file.
    pub fn with_message(message: MessageContent, file: FileContent) -> Self {
        Self { message, file, audio: Default::default(), relates_to: None }
    }
}

/// Audio content.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AudioContent {
    /// The duration of the video in milliseconds.
    #[serde(
        with = "ruma_common::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The waveform representation of the audio content.
    #[serde(default, skip_serializing_if = "Waveform::is_empty")]
    pub waveform: Waveform,
}

impl AudioContent {
    /// Creates a new empty `AudioContent`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The waveform representation of audio content.
///
/// Must include between 30 and 120 `Amplitude`s.
///
/// To build this, use the `TryFrom` implementations.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(try_from = "WaveformSerDeHelper")]
pub struct Waveform(Vec<Amplitude>);

impl Waveform {
    /// The smallest number of values contained in a `Waveform`.
    pub const MIN_LENGTH: usize = 30;

    /// The largest number of values contained in a `Waveform`.
    pub const MAX_LENGTH: usize = 120;

    /// The amplitudes of this `Waveform`.
    pub fn amplitudes(&self) -> &[Amplitude] {
        &self.0
    }

    /// Whether this `Waveform` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// An error encountered when trying to convert to a `Waveform`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum WaveformError {
    /// There are more than [`Waveform::MAX`] values.
    #[error("too many values")]
    TooManyValues,
    /// There are less that [`Waveform::MIN`] values.
    #[error("not enough values")]
    NotEnoughValues,
}

impl TryFrom<Vec<Amplitude>> for Waveform {
    type Error = WaveformError;

    fn try_from(value: Vec<Amplitude>) -> Result<Self, Self::Error> {
        if value.len() < Self::MIN_LENGTH {
            Err(WaveformError::NotEnoughValues)
        } else if value.len() > Self::MAX_LENGTH {
            Err(WaveformError::TooManyValues)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&[Amplitude]> for Waveform {
    type Error = WaveformError;

    fn try_from(value: &[Amplitude]) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

/// The amplitude of a waveform sample.
///
/// Must be an integer between 0 and 1024.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Amplitude(UInt);

impl Amplitude {
    /// The smallest value that can be represented by this type, 0.
    pub const MIN: u16 = 0;

    /// The largest value that can be represented by this type, 1024.
    pub const MAX: u16 = 1024;

    /// Creates a new `Amplitude` with the given value.
    ///
    /// It will saturate if it is bigger than [`Amplitude::MAX`].
    pub fn new(value: u16) -> Self {
        Self(value.min(Self::MAX).into())
    }

    /// The value of this `Amplitude`.
    pub fn value(&self) -> UInt {
        self.0
    }
}

impl From<u16> for Amplitude {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}
