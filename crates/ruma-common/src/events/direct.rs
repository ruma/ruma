//! Types for the [`m.direct`] event.
//!
//! [`m.direct`]: https://spec.matrix.org/latest/client-server-api/#mdirect

use std::{
    collections::{btree_map, BTreeMap},
    ops::{Deref, DerefMut},
};

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{OwnedRoomId, OwnedUserId};

/// The content of an `m.direct` event.
///
/// A mapping of `UserId`s to a list of `RoomId`s which are considered *direct* for that particular
/// user.
///
/// Informs the client about the rooms that are considered direct by a user.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.direct", kind = GlobalAccountData)]
pub struct DirectEventContent(pub BTreeMap<OwnedUserId, Vec<OwnedRoomId>>);

impl Deref for DirectEventContent {
    type Target = BTreeMap<OwnedUserId, Vec<OwnedRoomId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DirectEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for DirectEventContent {
    type Item = (OwnedUserId, Vec<OwnedRoomId>);
    type IntoIter = btree_map::IntoIter<OwnedUserId, Vec<OwnedRoomId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(OwnedUserId, Vec<OwnedRoomId>)> for DirectEventContent {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (OwnedUserId, Vec<OwnedRoomId>)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use std::collections::BTreeMap;

    use crate::{server_name, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DirectEvent, DirectEventContent};

    #[test]
    fn serialization() {
        let mut content = DirectEventContent(BTreeMap::new());
        let server_name = server_name!("ruma.io");
        let alice = UserId::new(server_name);
        let rooms = vec![RoomId::new(server_name)];

        content.insert(alice.clone(), rooms.clone());

        let json_data = json!({
            alice.to_string(): rooms,
        });

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let server_name = server_name!("ruma.io");
        let alice = UserId::new(server_name);
        let rooms = vec![RoomId::new(server_name), RoomId::new(server_name)];

        let json_data = json!({
            "content": {
                alice.to_string(): vec![rooms[0].to_string(), rooms[1].to_string()],
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value(json_data).unwrap();
        let direct_rooms = event.content.get(&alice).unwrap();

        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));
    }
}
