//! Modules for events in the `m.poll` namespace ([MSC3381]).
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381

use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use indexmap::IndexMap;
use js_int::{uint, UInt};
use ruma_common::{MilliSecondsSinceUnixEpoch, UserId};

use self::{start::PollContentBlock, unstable_start::UnstablePollStartContentBlock};

pub mod end;
pub mod response;
pub mod start;
pub mod unstable_end;
pub mod unstable_response;
pub mod unstable_start;

/// The data from a poll response necessary to compile poll results.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct PollResponseData<'a> {
    /// The sender of the response.
    pub sender: &'a UserId,

    /// The time of creation of the response on the originating server.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The selections/answers of the response.
    pub selections: &'a [String],
}

/// Generate the current results with the given poll and responses.
///
/// If the `end_timestamp` is provided, any response with an `origin_server_ts` after that timestamp
/// is ignored. If it is not provided, `MilliSecondsSinceUnixEpoch::now()` will be used instead.
///
/// This method will handle invalid responses, or several response from the same user so all
/// responses to the poll should be provided.
///
/// Returns a map of answer ID to a set of user IDs that voted for them. When using `.iter()` or
/// `.into_iter()` on the map, the results are sorted from the highest number of votes to the
/// lowest.
pub fn compile_poll_results<'a>(
    poll: &'a PollContentBlock,
    responses: impl IntoIterator<Item = PollResponseData<'a>>,
    end_timestamp: Option<MilliSecondsSinceUnixEpoch>,
) -> IndexMap<&'a str, BTreeSet<&'a UserId>> {
    let answer_ids = poll.answers.iter().map(|a| a.id.as_str()).collect();
    let users_selections =
        filter_selections(answer_ids, poll.max_selections, responses, end_timestamp);

    aggregate_results(poll.answers.iter().map(|a| a.id.as_str()), users_selections)
}

/// Generate the current results with the given unstable poll and responses.
///
/// If the `end_timestamp` is provided, any response with an `origin_server_ts` after that timestamp
/// is ignored. If it is not provided, `MilliSecondsSinceUnixEpoch::now()` will be used instead.
///
/// This method will handle invalid responses, or several response from the same user so all
/// responses to the poll should be provided.
///
/// Returns a map of answer ID to a set of user IDs that voted for them. When using `.iter()` or
/// `.into_iter()` on the map, the results are sorted from the highest number of votes to the
/// lowest.
pub fn compile_unstable_poll_results<'a>(
    poll: &'a UnstablePollStartContentBlock,
    responses: impl IntoIterator<Item = PollResponseData<'a>>,
    end_timestamp: Option<MilliSecondsSinceUnixEpoch>,
) -> IndexMap<&'a str, BTreeSet<&'a UserId>> {
    let answer_ids = poll.answers.iter().map(|a| a.id.as_str()).collect();
    let users_selections =
        filter_selections(answer_ids, poll.max_selections, responses, end_timestamp);

    aggregate_results(poll.answers.iter().map(|a| a.id.as_str()), users_selections)
}

/// Validate the selections of a response.
fn validate_selections<'a>(
    answer_ids: &BTreeSet<&str>,
    max_selections: UInt,
    selections: &'a [String],
) -> Option<impl Iterator<Item = &'a str>> {
    // Vote is spoiled if any answer is unknown.
    if selections.iter().any(|s| !answer_ids.contains(s.as_str())) {
        return None;
    }

    // Fallback to the maximum value for usize because we can't have more selections than that
    // in memory.
    let max_selections: usize = max_selections.try_into().unwrap_or(usize::MAX);

    Some(selections.iter().take(max_selections).map(Deref::deref))
}

fn filter_selections<'a>(
    answer_ids: BTreeSet<&str>,
    max_selections: UInt,
    responses: impl IntoIterator<Item = PollResponseData<'a>>,
    end_timestamp: Option<MilliSecondsSinceUnixEpoch>,
) -> BTreeMap<&'a UserId, (MilliSecondsSinceUnixEpoch, Option<impl Iterator<Item = &'a str>>)> {
    responses
        .into_iter()
        .filter(|ev| {
            // Filter out responses after the end_timestamp.
            end_timestamp.map_or(true, |end_ts| ev.origin_server_ts <= end_ts)
        })
        .fold(BTreeMap::new(), |mut acc, data| {
            let response =
                acc.entry(data.sender).or_insert((MilliSecondsSinceUnixEpoch(uint!(0)), None));

            // Only keep the latest selections for each user.
            if response.0 < data.origin_server_ts {
                *response = (
                    data.origin_server_ts,
                    validate_selections(&answer_ids, max_selections, data.selections),
                );
            }

            acc
        })
}

/// Aggregate the given selections by answer.
fn aggregate_results<'a>(
    answers: impl Iterator<Item = &'a str>,
    users_selections: BTreeMap<
        &'a UserId,
        (MilliSecondsSinceUnixEpoch, Option<impl Iterator<Item = &'a str>>),
    >,
) -> IndexMap<&'a str, BTreeSet<&'a UserId>> {
    let mut results = IndexMap::from_iter(answers.into_iter().map(|a| (a, BTreeSet::new())));

    for (user, (_, selections)) in users_selections {
        if let Some(selections) = selections {
            for selection in selections {
                results
                    .get_mut(selection)
                    .expect("validated selections should only match possible answers")
                    .insert(user);
            }
        }
    }

    results.sort_by(|_, a, _, b| b.len().cmp(&a.len()));

    results
}

/// Generate the fallback text representation of a poll end event.
///
/// This is a sentence that lists the top answers for the given results, in english. It is used to
/// generate a valid poll end event when using
/// `OriginalSync(Unstable)PollStartEvent::compile_results()`.
///
/// `answers` is an iterator of `(answer ID, answer plain text representation)` and `results` is an
/// iterator of `(answer ID, count)` ordered in descending order.
fn generate_poll_end_fallback_text<'a>(
    answers: &[(&'a str, &'a str)],
    results: impl Iterator<Item = (&'a str, usize)>,
) -> String {
    let mut top_answers = Vec::new();
    let mut top_count = 0;

    for (id, count) in results {
        if count >= top_count {
            top_answers.push(id);
            top_count = count;
        } else {
            break;
        }
    }

    let top_answers_text = top_answers
        .into_iter()
        .map(|id| {
            answers
                .iter()
                .find(|(a_id, _)| *a_id == id)
                .expect("top answer ID should be a valid answer ID")
                .1
        })
        .collect::<Vec<_>>();

    // Construct the plain text representation.
    match top_answers_text.len() {
        0 => "The poll has closed with no top answer".to_owned(),
        1 => {
            format!("The poll has closed. Top answer: {}", top_answers_text[0])
        }
        _ => {
            let answers = top_answers_text.join(", ");
            format!("The poll has closed. Top answers: {answers}")
        }
    }
}
