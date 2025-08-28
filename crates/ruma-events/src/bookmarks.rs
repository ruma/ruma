//! Types for the [`m.push_rules`] event.
//!
//! [`m.push_rules`]: https://spec.matrix.org/latest/client-server-api/#mpush_rules

use std::collections::BTreeMap;

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// TODO
pub type BookmarkMap = BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, BookmarkInfo>>;

/// The content of an `m.bookmarks` event.
///
/// Describes all bookmarks of a user.
#[derive(Clone, Debug, Deserialize, Serialize, Default, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.bookmarks", kind = GlobalAccountData)]
pub struct BookmarksEventContent {
    /// TODO
    pub map: BookmarkMap,
}

impl BookmarksEventContent {
    /// Creates a new [`BookmarksEventContent`] with the given bookmarks.
    pub fn new(map: BookmarkMap) -> Self {
        Self { map }
    }
}

/// TODO
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookmarkInfo {
    /// TODO
    pub created_at: MilliSecondsSinceUnixEpoch,
    /// TODO
    pub annotation: Option<String>,
    /// TODO
    pub reminder: Option<MilliSecondsSinceUnixEpoch>,
}
