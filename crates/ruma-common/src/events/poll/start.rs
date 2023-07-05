//! Types for the `m.poll.start` event.

use std::ops::Deref;

use js_int::{uint, UInt};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod poll_answers_serde;

use poll_answers_serde::PollAnswersDeHelper;

use crate::{events::message::TextContentBlock, serde::StringEnum, PrivOwnedStr};

use super::{
    compile_poll_results,
    end::{PollEndEventContent, PollResultsContentBlock},
    response::OriginalSyncPollResponseEvent,
};

/// The payload for a poll start event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.poll.start", kind = MessageLike)]
pub struct PollStartEventContent {
    /// The poll content of the message.
    #[serde(rename = "m.poll")]
    pub poll: PollContentBlock,

    /// Text representation of the message, for clients that don't support polls.
    #[serde(rename = "m.text")]
    pub text: TextContentBlock,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "crate::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,
}

impl PollStartEventContent {
    /// Creates a new `PollStartEventContent` with the given fallback representation and poll
    /// content.
    pub fn new(text: TextContentBlock, poll: PollContentBlock) -> Self {
        Self {
            poll,
            text,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
        }
    }

    /// Creates a new `PollStartEventContent` with the given plain text fallback
    /// representation and poll content.
    pub fn with_plain_text(plain_text: impl Into<String>, poll: PollContentBlock) -> Self {
        Self {
            poll,
            text: TextContentBlock::plain(plain_text),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
        }
    }
}

impl OriginalSyncPollStartEvent {
    /// Compile the results for this poll with the given response into a `PollEndEventContent`.
    ///
    /// It generates a default text representation of the results in English.
    ///
    /// This uses [`compile_poll_results()`] internally.
    pub fn compile_results<'a>(
        &'a self,
        responses: impl IntoIterator<Item = &'a OriginalSyncPollResponseEvent>,
    ) -> PollEndEventContent {
        let computed = compile_poll_results(&self.content.poll, responses, None);

        // Construct the results and get the top answer(s).
        let mut top_answers = Vec::new();
        let mut top_count = uint!(0);

        let results =
            PollResultsContentBlock::from_iter(computed.into_iter().map(|(id, users)| {
                let count = users.len().try_into().unwrap_or(UInt::MAX);

                if count >= top_count {
                    top_answers.push(id);
                    top_count = count;
                }

                (id.to_owned(), count)
            }));

        // Get the text representation of the best answers.
        let top_answers_text = top_answers
            .into_iter()
            .map(|id| {
                let text = &self
                    .content
                    .poll
                    .answers
                    .iter()
                    .find(|a| a.id == id)
                    .expect("top answer ID should be a valid answer ID")
                    .text;
                // Use the plain text representation, fallback to the first text representation or
                // to the answer ID.
                text.find_plain()
                    .or_else(|| text.get(0).map(|t| t.body.as_ref()))
                    .unwrap_or(id)
                    .to_owned()
            })
            .collect::<Vec<_>>();

        // Construct the plain text representation.
        let plain_text = match top_answers_text.len() {
            l if l > 1 => {
                let answers = top_answers_text.join(", ");
                format!("The poll has closed. Top answers: {answers}")
            }
            l if l == 1 => {
                format!("The poll has closed. Top answer: {}", top_answers_text[0])
            }
            _ => "The poll has closed with no top answer".to_owned(),
        };

        let mut end = PollEndEventContent::with_plain_text(plain_text, self.event_id.clone());
        end.poll_results = Some(results);

        end
    }
}

/// A block for poll content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollContentBlock {
    /// The question of the poll.
    pub question: PollQuestion,

    /// The kind of the poll.
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub kind: PollKind,

    /// The maximum number of responses a user is able to select.
    ///
    /// Must be greater or equal to `1`.
    ///
    /// Defaults to `1`.
    #[serde(
        default = "PollContentBlock::default_max_selections",
        skip_serializing_if = "PollContentBlock::max_selections_is_default"
    )]
    pub max_selections: UInt,

    /// The possible answers to the poll.
    pub answers: PollAnswers,
}

impl PollContentBlock {
    /// Creates a new `PollStartContent` with the given question and answers.
    pub fn new(question: TextContentBlock, answers: PollAnswers) -> Self {
        Self {
            question: question.into(),
            kind: Default::default(),
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
    #[serde(rename = "m.text")]
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
    #[ruma_enum(rename = "m.undisclosed")]
    Undisclosed,

    /// The votes are visible up until and including when the poll is closed.
    #[ruma_enum(rename = "m.disclosed")]
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

impl Deref for PollAnswers {
    type Target = [PollAnswer];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Poll answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PollAnswer {
    /// The ID of the answer.
    ///
    /// This must be unique among the answers of a poll.
    #[serde(rename = "m.id")]
    pub id: String,

    /// The text representation of the answer.
    #[serde(rename = "m.text")]
    pub text: TextContentBlock,
}

impl PollAnswer {
    /// Creates a new `PollAnswer` with the given id and text representation.
    pub fn new(id: String, text: TextContentBlock) -> Self {
        Self { id, text }
    }
}
