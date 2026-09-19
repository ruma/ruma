//! Types for the [`m.direct`] event.
//!
//! [`m.direct`]: https://spec.matrix.org/v1.19/client-server-api/#mdirect

use std::{
    collections::{BTreeMap, btree_map},
    ops::{Deref, DerefMut},
};

pub use ruma_common::DirectUserIdentifier;
use ruma_common::RoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.direct` event.
///
/// A mapping of `DirectUserIdentifier`s to a list of `RoomId`s which are considered *direct*
/// for that particular user.
///
/// Informs the client about the rooms that are considered direct by a user.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.direct", kind = GlobalAccountData)]
pub struct DirectEventContent(pub BTreeMap<DirectUserIdentifier, Vec<RoomId>>);

impl Deref for DirectEventContent {
    type Target = BTreeMap<DirectUserIdentifier, Vec<RoomId>>;

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
    type Item = (DirectUserIdentifier, Vec<RoomId>);
    type IntoIter = btree_map::IntoIter<DirectUserIdentifier, Vec<RoomId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(DirectUserIdentifier, Vec<RoomId>)> for DirectEventContent {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (DirectUserIdentifier, Vec<RoomId>)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_common::{canonical_json::assert_to_canonical_json_eq, room_id, user_id};
    use serde_json::{from_value as from_json_value, json};

    use super::{DirectEvent, DirectEventContent};

    #[test]
    fn serialization() {
        let mut content = DirectEventContent(BTreeMap::new());
        let alice = user_id!("@alice:ruma.io");
        let alice_mail = "alice@ruma.io";
        let rooms = vec![room_id!("!1:ruma.io")];
        let mail_rooms = vec![room_id!("!3:ruma.io")];

        content.insert(alice.clone().into(), rooms.clone());
        content.insert(alice_mail.into(), mail_rooms.clone());

        let json_data = json!({
            alice: rooms,
            alice_mail: mail_rooms,
        });

        assert_to_canonical_json_eq!(content, json_data);
    }

    #[test]
    fn deserialization() {
        let alice = user_id!("@alice:ruma.io");
        let alice_mail = "alice@ruma.io";
        let rooms = vec![room_id!("!1:ruma.io"), room_id!("!2:ruma.io")];
        let mail_rooms = vec![room_id!("!3:ruma.io")];

        let json_data = json!({
            "content": {
                &alice: rooms,
                alice_mail: mail_rooms,
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value(json_data).unwrap();

        let direct_rooms = event.content.get(alice.as_str()).unwrap();
        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));

        let email_direct_rooms = event.content.get(alice_mail).unwrap();
        assert!(email_direct_rooms.contains(&mail_rooms[0]));
    }
}
