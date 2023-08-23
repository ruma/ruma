//! Types for the `org.matrix.msc3381.poll.start` event, the unstable version of `m.poll.start`.

use std::ops::Deref;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod unstable_poll_answers_serde;
mod unstable_poll_kind_serde;

use self::unstable_poll_answers_serde::UnstablePollAnswersDeHelper;
use super::{
    compile_unstable_poll_results, generate_poll_end_fallback_text,
    start::{PollAnswers, PollAnswersError, PollContentBlock, PollKind},
    unstable_end::UnstablePollEndEventContent,
    PollResponseData,
};
use crate::{events::room::message::Relation, MilliSecondsSinceUnixEpoch};

/// The payload for an unstable poll start event.
///
/// This is the event content that should be sent for room versions that don't support extensible
/// events. As of Matrix 1.7, none of the stable room versions (1 through 10) support extensible
/// events.
///
/// To send a poll start event for a room version that supports extensible events, use
/// [`PollStartEventContent`].
///
/// [`PollStartEventContent`]: super::start::PollStartEventContent
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.start", kind = MessageLike, without_relation)]
pub struct UnstablePollStartEventContent {
    /// The poll content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.start")]
    pub poll_start: UnstablePollStartContentBlock,

    /// Text representation of the message, for clients that don't support polls.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: Option<String>,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::events::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<UnstablePollStartEventContentWithoutRelation>>,
}

impl UnstablePollStartEventContent {
    /// Creates a new `PollStartEventContent` with the given poll content.
    pub fn new(poll_start: UnstablePollStartContentBlock) -> Self {
        Self { poll_start, text: None, relates_to: None }
    }

    /// Creates a new `PollStartEventContent` with the given plain text fallback
    /// representation and poll content.
    pub fn plain_text(text: impl Into<String>, poll_start: UnstablePollStartContentBlock) -> Self {
        Self { poll_start, text: Some(text.into()), relates_to: None }
    }
}

impl OriginalSyncUnstablePollStartEvent {
    /// Compile the results for this poll with the given response into an
    /// `UnstablePollEndEventContent`.
    ///
    /// It generates a default text representation of the results in English.
    ///
    /// This uses [`compile_unstable_poll_results()`] internally.
    pub fn compile_results<'a>(
        &'a self,
        responses: impl IntoIterator<Item = PollResponseData<'a>>,
    ) -> UnstablePollEndEventContent {
        let full_results = compile_unstable_poll_results(
            &self.content.poll_start,
            responses,
            Some(MilliSecondsSinceUnixEpoch::now()),
        );
        let results =
            full_results.into_iter().map(|(id, users)| (id, users.len())).collect::<Vec<_>>();

        // Get the text representation of the best answers.
        let answers = self
            .content
            .poll_start
            .answers
            .iter()
            .map(|a| (a.id.as_str(), a.text.as_str()))
            .collect::<Vec<_>>();
        let plain_text = generate_poll_end_fallback_text(&answers, results.into_iter());

        UnstablePollEndEventContent::new(plain_text, self.event_id.clone())
    }
}

/// An unstable block for poll start content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstablePollStartContentBlock {
    /// The question of the poll.
    pub question: UnstablePollQuestion,

    /// The kind of the poll.
    #[serde(default, with = "unstable_poll_kind_serde")]
    pub kind: PollKind,

    /// The maximum number of responses a user is able to select.
    ///
    /// Must be greater or equal to `1`.
    ///
    /// Defaults to `1`.
    #[serde(default = "PollContentBlock::default_max_selections")]
    pub max_selections: UInt,

    /// The possible answers to the poll.
    pub answers: UnstablePollAnswers,
}

impl UnstablePollStartContentBlock {
    /// Creates a new `PollStartContent` with the given question and answers.
    pub fn new(question: impl Into<String>, answers: UnstablePollAnswers) -> Self {
        Self {
            question: UnstablePollQuestion::new(question),
            kind: Default::default(),
            max_selections: PollContentBlock::default_max_selections(),
            answers,
        }
    }
}

/// An unstable poll question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstablePollQuestion {
    /// The text representation of the question.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: String,
}

impl UnstablePollQuestion {
    /// Creates a new `UnstablePollQuestion` with the given plain text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// The unstable answers to a poll.
///
/// Must include between 1 and 20 `UnstablePollAnswer`s.
///
/// To build this, use one of the `TryFrom` implementations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "UnstablePollAnswersDeHelper")]
pub struct UnstablePollAnswers(Vec<UnstablePollAnswer>);

impl TryFrom<Vec<UnstablePollAnswer>> for UnstablePollAnswers {
    type Error = PollAnswersError;

    fn try_from(value: Vec<UnstablePollAnswer>) -> Result<Self, Self::Error> {
        if value.len() < PollAnswers::MIN_LENGTH {
            Err(PollAnswersError::NotEnoughValues)
        } else if value.len() > PollAnswers::MAX_LENGTH {
            Err(PollAnswersError::TooManyValues)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&[UnstablePollAnswer]> for UnstablePollAnswers {
    type Error = PollAnswersError;

    fn try_from(value: &[UnstablePollAnswer]) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl Deref for UnstablePollAnswers {
    type Target = [UnstablePollAnswer];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Unstable poll answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnstablePollAnswer {
    /// The ID of the answer.
    ///
    /// This must be unique among the answers of a poll.
    pub id: String,

    /// The text representation of the answer.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: String,
}

impl UnstablePollAnswer {
    /// Creates a new `PollAnswer` with the given id and text representation.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self { id: id.into(), text: text.into() }
    }
}
