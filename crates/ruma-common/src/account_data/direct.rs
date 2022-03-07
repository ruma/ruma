//! Types for the [`m.direct`] object.
//!
//! [`m.direct`]: https://spec.matrix.org/v1.2/client-server-api/#mdirect

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use ruma_identifiers::{RoomId, UserId};
use ruma_macros::AccountDataContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.direct` object.
///
/// A mapping of `UserId`s to a list of `RoomId`s which are considered *direct* for that particular
/// user.
///
/// Informs the client about the rooms that are considered direct by a user.
#[derive(Clone, Debug, Deserialize, Serialize, AccountDataContent)]
#[allow(clippy::exhaustive_structs)]
#[account_data(type = "m.direct", kind = Global)]
pub struct DirectContent(pub BTreeMap<Box<UserId>, Vec<Box<RoomId>>>);

impl Deref for DirectContent {
    type Target = BTreeMap<Box<UserId>, Vec<Box<RoomId>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DirectContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_identifiers::{server_name, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{Direct, DirectContent};

    #[test]
    fn serialization() {
        let mut content = DirectContent(BTreeMap::new());
        let server_name = server_name!("ruma.io");
        let alice = UserId::new(server_name);
        let room = vec![RoomId::new(server_name)];

        content.insert(alice.clone(), room.clone());

        let object = Direct { content };
        let json_data = json!({
            "content": {
                alice.to_string(): vec![room[0].to_string()],
            },
            "type": "m.direct"
        });

        assert_eq!(to_json_value(&object).unwrap(), json_data);
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

        let object: Direct = from_json_value(json_data).unwrap();
        let direct_rooms = object.content.get(&alice).unwrap();

        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));
    }
}
