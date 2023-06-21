//! Modules for events in the `m.poll` namespace ([MSC3381]).
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use js_int::uint;

use crate::{MilliSecondsSinceUnixEpoch, UserId};

use self::{response::OriginalSyncPollResponseEvent, start::PollContentBlock};

pub mod end;
pub mod response;
pub mod start;

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
    responses: impl IntoIterator<Item = &'a OriginalSyncPollResponseEvent>,
    end_timestamp: Option<MilliSecondsSinceUnixEpoch>,
) -> IndexMap<&'a str, BTreeSet<&'a UserId>> {
    let end_ts = end_timestamp.unwrap_or_else(MilliSecondsSinceUnixEpoch::now);

    let users_selections = responses
        .into_iter()
        .filter(|ev| {
            // Filter out responses after the end_timestamp.
            ev.origin_server_ts <= end_ts
        })
        .fold(BTreeMap::new(), |mut acc, ev| {
            let response =
                acc.entry(&*ev.sender).or_insert((MilliSecondsSinceUnixEpoch(uint!(0)), None));

            // Only keep the latest selections for each user.
            if response.0 < ev.origin_server_ts {
                *response = (ev.origin_server_ts, ev.content.selections.validate(poll));
            }

            acc
        });

    // Aggregate the selections by answer.
    let mut results =
        IndexMap::from_iter(poll.answers.iter().map(|a| (a.id.as_str(), BTreeSet::new())));

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
