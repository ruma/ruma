//! `Deserialize` helpers for unstable poll answers (MSC3381).

use serde::Deserialize;

use crate::events::poll::start::{PollAnswers, PollAnswersError};

use super::{UnstablePollAnswer, UnstablePollAnswers};

#[derive(Debug, Default, Deserialize)]
pub(crate) struct UnstablePollAnswersDeHelper(Vec<UnstablePollAnswer>);

impl TryFrom<UnstablePollAnswersDeHelper> for UnstablePollAnswers {
    type Error = PollAnswersError;

    fn try_from(helper: UnstablePollAnswersDeHelper) -> Result<Self, Self::Error> {
        let mut answers = helper.0;
        answers.truncate(PollAnswers::MAX_LENGTH);
        UnstablePollAnswers::try_from(answers)
    }
}
