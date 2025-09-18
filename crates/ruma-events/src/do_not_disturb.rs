//! Types for the [`dm.filament.do_not_disturb`] event.
//!
//! [`dm.filament.do_not_disturb`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4359

use std::collections::BTreeMap;

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of a `dm.filament.do_not_disturb` event.
///
/// A list of rooms in "Do not Disturb" mode.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "dm.filament.do_not_disturb", kind = GlobalAccountData)]
pub struct DoNotDisturbEventContent {
    /// A map of rooms in which to inhibit notifications.
    ///
    /// As [`DoNotDisturbRoom`] is currently empty, only the room IDs are useful and
    /// can be accessed with the `.keys()` and `into_keys()` iterators.
    pub rooms: BTreeMap<DoNotDisturbRoomKey, DoNotDisturbRoom>,
}

impl DoNotDisturbEventContent {
    /// Creates a new `DoNotDisturbEventContent` from the given map of [`DoNotDisturbRoom`]s.
    pub fn new(rooms: BTreeMap<DoNotDisturbRoomKey, DoNotDisturbRoom>) -> Self {
        Self { rooms }
    }
}

impl FromIterator<DoNotDisturbRoomKey> for DoNotDisturbEventContent {
    fn from_iter<T: IntoIterator<Item = DoNotDisturbRoomKey>>(iter: T) -> Self {
        Self::new(iter.into_iter().map(|key| (key, DoNotDisturbRoom {})).collect())
    }
}

impl FromIterator<OwnedRoomId> for DoNotDisturbEventContent {
    fn from_iter<T: IntoIterator<Item = OwnedRoomId>>(iter: T) -> Self {
        iter.into_iter().map(DoNotDisturbRoomKey::SingleRoom).collect()
    }
}

impl Extend<DoNotDisturbRoomKey> for DoNotDisturbEventContent {
    fn extend<T: IntoIterator<Item = DoNotDisturbRoomKey>>(&mut self, iter: T) {
        self.rooms.extend(iter.into_iter().map(|key| (key, DoNotDisturbRoom {})));
    }
}

impl Extend<OwnedRoomId> for DoNotDisturbEventContent {
    fn extend<T: IntoIterator<Item = OwnedRoomId>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(DoNotDisturbRoomKey::SingleRoom));
    }
}

/// The key for a "Do not Disturb" setting.
///
/// This either matches a single room or all rooms.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum DoNotDisturbRoomKey {
    /// Match any room.
    #[serde(rename = "*")]
    AllRooms,

    /// Match a single room based on its room ID.
    #[serde(untagged)]
    SingleRoom(OwnedRoomId),
}

/// Details about a room in "Do not Disturb" mode.
///
/// This is currently empty.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DoNotDisturbRoom {}

impl DoNotDisturbRoom {
    /// Creates an empty `DoNotDisturbRoom`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use ruma_common::owned_room_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::DoNotDisturbEventContent;
    use crate::{do_not_disturb::DoNotDisturbRoomKey, AnyGlobalAccountDataEvent};

    #[test]
    fn serialization_with_single_room() {
        let do_not_disturb_room_list: DoNotDisturbEventContent =
            vec![owned_room_id!("!foo:bar.baz")].into_iter().collect();

        let json = json!({
            "rooms": {
                "!foo:bar.baz": {}
            },
        });

        assert_eq!(to_json_value(do_not_disturb_room_list).unwrap(), json);
    }

    #[test]
    fn serialization_with_all_rooms() {
        let do_not_disturb_room_list = DoNotDisturbEventContent::new(BTreeMap::from([(
            DoNotDisturbRoomKey::AllRooms,
            Default::default(),
        )]));

        let json = json!({
            "rooms": {
                "*": {}
            },
        });

        assert_eq!(to_json_value(do_not_disturb_room_list).unwrap(), json);
    }

    #[test]
    fn deserialization_with_single_room() {
        let json = json!({
            "content": {
                "rooms": {
                    "!foo:bar.baz": {}
                }
            },
            "type": "dm.filament.do_not_disturb"
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::DoNotDisturb(ev))
        );
        assert_eq!(
            ev.content.rooms.keys().collect::<Vec<_>>(),
            vec![&DoNotDisturbRoomKey::SingleRoom(owned_room_id!("!foo:bar.baz"))]
        );
    }

    #[test]
    fn deserialization_with_all_room() {
        let json = json!({
            "content": {
                "rooms": {
                    "*": {}
                }
            },
            "type": "dm.filament.do_not_disturb"
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::DoNotDisturb(ev))
        );
        assert_eq!(
            ev.content.rooms.keys().collect::<Vec<_>>(),
            vec![&DoNotDisturbRoomKey::AllRooms]
        );
    }
}
