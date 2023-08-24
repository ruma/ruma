//! Types for the `org.matrix.msc3381.poll.start` event, the unstable version of `m.poll.start`.

use std::ops::Deref;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod content_serde;
mod unstable_poll_answers_serde;
mod unstable_poll_kind_serde;

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedEventId};

use self::unstable_poll_answers_serde::UnstablePollAnswersDeHelper;
use super::{
    compile_unstable_poll_results, generate_poll_end_fallback_text,
    start::{PollAnswers, PollAnswersError, PollContentBlock, PollKind},
    unstable_end::UnstablePollEndEventContent,
    PollResponseData,
};
use crate::{
    relation::Replacement, room::message::RelationWithoutReplacement, EventContent,
    MessageLikeEventContent, MessageLikeEventType, RedactContent, RedactedMessageLikeEventContent,
    StaticEventContent,
};

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
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3381.poll.start", kind = MessageLike, custom_redacted)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum UnstablePollStartEventContent {
    /// A new poll start event.
    New(NewUnstablePollStartEventContent),

    /// A replacement poll start event.
    Replacement(ReplacementUnstablePollStartEventContent),
}

impl UnstablePollStartEventContent {
    /// Get the poll start content of this event content.
    pub fn poll_start(&self) -> &UnstablePollStartContentBlock {
        match self {
            Self::New(c) => &c.poll_start,
            Self::Replacement(c) => &c.relates_to.new_content.poll_start,
        }
    }
}

impl RedactContent for UnstablePollStartEventContent {
    type Redacted = RedactedUnstablePollStartEventContent;

    fn redact(self, _version: &crate::RoomVersionId) -> Self::Redacted {
        RedactedUnstablePollStartEventContent::default()
    }
}

impl From<NewUnstablePollStartEventContent> for UnstablePollStartEventContent {
    fn from(value: NewUnstablePollStartEventContent) -> Self {
        Self::New(value)
    }
}

impl From<ReplacementUnstablePollStartEventContent> for UnstablePollStartEventContent {
    fn from(value: ReplacementUnstablePollStartEventContent) -> Self {
        Self::Replacement(value)
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
        let poll_start = self.content.poll_start();

        let full_results = compile_unstable_poll_results(
            poll_start,
            responses,
            Some(MilliSecondsSinceUnixEpoch::now()),
        );
        let results =
            full_results.into_iter().map(|(id, users)| (id, users.len())).collect::<Vec<_>>();

        // Get the text representation of the best answers.
        let answers =
            poll_start.answers.iter().map(|a| (a.id.as_str(), a.text.as_str())).collect::<Vec<_>>();
        let plain_text = generate_poll_end_fallback_text(&answers, results.into_iter());

        UnstablePollEndEventContent::new(plain_text, self.event_id.clone())
    }
}

/// A new unstable poll start event.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NewUnstablePollStartEventContent {
    /// The poll content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.start")]
    pub poll_start: UnstablePollStartContentBlock,

    /// Text representation of the message, for clients that don't support polls.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: Option<String>,

    /// Information about related messages.
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelationWithoutReplacement>,
}

impl NewUnstablePollStartEventContent {
    /// Creates a `NewUnstablePollStartEventContent` with the given poll content.
    pub fn new(poll_start: UnstablePollStartContentBlock) -> Self {
        Self { poll_start, text: None, relates_to: None }
    }

    /// Creates a `NewUnstablePollStartEventContent` with the given plain text fallback
    /// representation and poll content.
    pub fn plain_text(text: impl Into<String>, poll_start: UnstablePollStartContentBlock) -> Self {
        Self { poll_start, text: Some(text.into()), relates_to: None }
    }
}

impl EventContent for NewUnstablePollStartEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> Self::EventType {
        MessageLikeEventType::UnstablePollStart
    }
}

impl StaticEventContent for NewUnstablePollStartEventContent {
    const TYPE: &'static str = "org.matrix.msc3381.poll.start";
}

impl MessageLikeEventContent for NewUnstablePollStartEventContent {}

/// Form of [`NewUnstablePollStartEventContent`] without relation.
///
/// To construct this type, construct a [`NewUnstablePollStartEventContent`] and then use one of its
/// `::from()` / `.into()` methods.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NewUnstablePollStartEventContentWithoutRelation {
    /// The poll content of the message.
    #[serde(rename = "org.matrix.msc3381.poll.start")]
    pub poll_start: UnstablePollStartContentBlock,

    /// Text representation of the message, for clients that don't support polls.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: Option<String>,
}

impl From<NewUnstablePollStartEventContent> for NewUnstablePollStartEventContentWithoutRelation {
    fn from(value: NewUnstablePollStartEventContent) -> Self {
        let NewUnstablePollStartEventContent { poll_start, text, .. } = value;
        Self { poll_start, text }
    }
}

/// A replacement unstable poll start event.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReplacementUnstablePollStartEventContent {
    /// The poll content of the message.
    pub poll_start: Option<UnstablePollStartContentBlock>,

    /// Text representation of the message, for clients that don't support polls.
    pub text: Option<String>,

    /// Information about related messages.
    pub relates_to: Replacement<NewUnstablePollStartEventContentWithoutRelation>,
}

impl ReplacementUnstablePollStartEventContent {
    /// Creates a `ReplacementUnstablePollStartEventContent` with the given poll content that
    /// replaces the event with the given ID.
    ///
    /// The constructed content does not have a fallback by default.
    pub fn new(poll_start: UnstablePollStartContentBlock, replaces: OwnedEventId) -> Self {
        Self {
            poll_start: None,
            text: None,
            relates_to: Replacement {
                event_id: replaces,
                new_content: NewUnstablePollStartEventContent::new(poll_start).into(),
            },
        }
    }

    /// Creates a `ReplacementUnstablePollStartEventContent` with the given plain text fallback
    /// representation and poll content that replaces the event with the given ID.
    ///
    /// The constructed content does not have a fallback by default.
    pub fn plain_text(
        text: impl Into<String>,
        poll_start: UnstablePollStartContentBlock,
        replaces: OwnedEventId,
    ) -> Self {
        Self {
            poll_start: None,
            text: None,
            relates_to: Replacement {
                event_id: replaces,
                new_content: NewUnstablePollStartEventContent::plain_text(text, poll_start).into(),
            },
        }
    }
}

impl EventContent for ReplacementUnstablePollStartEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> Self::EventType {
        MessageLikeEventType::UnstablePollStart
    }
}

impl StaticEventContent for ReplacementUnstablePollStartEventContent {
    const TYPE: &'static str = "org.matrix.msc3381.poll.start";
}

impl MessageLikeEventContent for ReplacementUnstablePollStartEventContent {}

/// Redacted form of UnstablePollStartEventContent
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedUnstablePollStartEventContent {}

impl RedactedUnstablePollStartEventContent {
    /// Creates an empty RedactedUnstablePollStartEventContent.
    pub fn new() -> RedactedUnstablePollStartEventContent {
        Self::default()
    }
}

impl EventContent for RedactedUnstablePollStartEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> Self::EventType {
        MessageLikeEventType::UnstablePollStart
    }
}

impl StaticEventContent for RedactedUnstablePollStartEventContent {
    const TYPE: &'static str = "org.matrix.msc3381.poll.start";
}

impl RedactedMessageLikeEventContent for RedactedUnstablePollStartEventContent {}

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
