//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use serde::Deserialize;

use super::{PollAnswer, PollAnswers, PollAnswersError};

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PollAnswersDeHelper(Vec<PollAnswer>);

impl TryFrom<PollAnswersDeHelper> for PollAnswers {
    type Error = PollAnswersError;

    fn try_from(helper: PollAnswersDeHelper) -> Result<Self, Self::Error> {
        let mut answers = helper.0;
        answers.truncate(PollAnswers::MAX_LENGTH);
        PollAnswers::try_from(answers)
    }
}
