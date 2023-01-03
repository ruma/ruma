//! Types for the [`m.poll.start`] event.

use js_int::{uint, UInt};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod poll_answers_serde;

use poll_answers_serde::PollAnswersDeHelper;

use crate::{events::message::TextContentBlock, serde::StringEnum, PrivOwnedStr};

/// The payload for a poll start event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.start", alias = "m.poll.start", kind = MessageLike)]
pub struct PollStartEventContent {
    /// The poll start content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.start", alias = "m.poll.start")]
    pub poll_start: PollStartContent,

    /// Optional fallback text representation of the message, for clients that don't support polls.
    #[serde(
        rename = "org.matrix.msc1767.text",
        default,
        skip_serializing_if = "TextContentBlock::is_empty"
    )]
    pub text: TextContentBlock,
}

impl PollStartEventContent {
    /// Creates a new `PollStartEventContent` with the given poll start content.
    pub fn new(poll_start: PollStartContent) -> Self {
        Self { poll_start, text: Default::default() }
    }
}

/// Poll start content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollStartContent {
    /// The question of the poll.
    pub question: PollQuestion,

    /// The kind of the poll.
    #[serde(default)]
    pub kind: PollKind,

    /// The maximum number of responses a user is able to select.
    ///
    /// Must be greater or equal to `1`.
    ///
    /// Defaults to `1`.
    #[serde(
        default = "PollStartContent::default_max_selections",
        skip_serializing_if = "PollStartContent::max_selections_is_default"
    )]
    pub max_selections: UInt,

    /// The possible answers to the poll.
    pub answers: PollAnswers,
}

impl PollStartContent {
    /// Creates a new `PollStartContent` with the given question, kind, and answers.
    pub fn new(question: TextContentBlock, kind: PollKind, answers: PollAnswers) -> Self {
        Self {
            question: question.into(),
            kind,
            max_selections: Self::default_max_selections(),
            answers,
        }
    }

    fn default_max_selections() -> UInt {
        uint!(1)
    }

    fn max_selections_is_default(max_selections: &UInt) -> bool {
        max_selections == &Self::default_max_selections()
    }
}

/// The question of a poll.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollQuestion {
    /// The text representation of the question.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,
}

impl From<TextContentBlock> for PollQuestion {
    fn from(text: TextContentBlock) -> Self {
        Self { text }
    }
}

/// The kind of poll.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum PollKind {
    /// The results are revealed once the poll is closed.
    #[default]
    #[ruma_enum(rename = "org.matrix.msc3381.poll.undisclosed", alias = "m.poll.undisclosed")]
    Undisclosed,

    /// The votes are visible up until and including when the poll is closed.
    #[ruma_enum(rename = "org.matrix.msc3381.poll.disclosed", alias = "m.poll.disclosed")]
    Disclosed,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The answers to a poll.
///
/// Must include between 1 and 20 `PollAnswer`s.
///
/// To build this, use the `TryFrom` implementations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "PollAnswersDeHelper")]
pub struct PollAnswers(Vec<PollAnswer>);

impl PollAnswers {
    /// The smallest number of values contained in a `PollAnswers`.
    pub const MIN_LENGTH: usize = 1;

    /// The largest number of values contained in a `PollAnswers`.
    pub const MAX_LENGTH: usize = 20;

    /// The answers of this `PollAnswers`.
    pub fn answers(&self) -> &[PollAnswer] {
        &self.0
    }
}

/// An error encountered when trying to convert to a `PollAnswers`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PollAnswersError {
    /// There are more than [`PollAnswers::MAX_LENGTH`] values.
    #[error("too many values")]
    TooManyValues,
    /// There are less that [`PollAnswers::MIN_LENGTH`] values.
    #[error("not enough values")]
    NotEnoughValues,
}

impl TryFrom<Vec<PollAnswer>> for PollAnswers {
    type Error = PollAnswersError;

    fn try_from(value: Vec<PollAnswer>) -> Result<Self, Self::Error> {
        if value.len() < Self::MIN_LENGTH {
            Err(PollAnswersError::NotEnoughValues)
        } else if value.len() > Self::MAX_LENGTH {
            Err(PollAnswersError::TooManyValues)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&[PollAnswer]> for PollAnswers {
    type Error = PollAnswersError;

    fn try_from(value: &[PollAnswer]) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

/// Poll answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollAnswer {
    /// The ID of the answer.
    ///
    /// This must be unique among the answers of a poll.
    pub id: String,

    /// The text representation of the answer.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,
}

impl PollAnswer {
    /// Creates a new `PollAnswer` with the given id and text representation.
    pub fn new(id: String, text: TextContentBlock) -> Self {
        Self { id, text }
    }
}
