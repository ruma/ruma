//! Types for the [`m.direct`] event.
//!
//! [`m.direct`]: https://spec.matrix.org/latest/client-server-api/#mdirect

use std::{
    collections::{BTreeMap, btree_map},
    ops::{Deref, DerefMut},
};

use ruma_common::{IdParseError, RoomId, UserId};
use ruma_macros::{EventContent, ruma_id};
use serde::{Deserialize, Serialize};

/// An user identifier, it can be a [`UserId`] or a third-party identifier like an email or a phone
/// number.
///
/// There is no validation on this type, any string is allowed, but you can use
/// [`DirectUserIdentifier::as_user_id()`] to try to get a [`UserId`].
#[ruma_id]
pub struct DirectUserIdentifier;

impl DirectUserIdentifier {
    /// Get this `DirectUserIdentifier` as a [`UserId`] if it is one.
    pub fn as_user_id(&self) -> Option<UserId> {
        UserId::try_from(self).ok()
    }
}

impl TryFrom<DirectUserIdentifier> for UserId {
    type Error = IdParseError;

    fn try_from(value: DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&DirectUserIdentifier> for UserId {
    type Error = IdParseError;

    fn try_from(value: &DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl From<UserId> for DirectUserIdentifier {
    fn from(value: UserId) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&UserId> for DirectUserIdentifier {
    fn from(value: &UserId) -> Self {
        Self::from(value.as_str())
    }
}

impl PartialEq<UserId> for DirectUserIdentifier {
    fn eq(&self, other: &UserId) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<DirectUserIdentifier> for UserId {
    fn eq(&self, other: &DirectUserIdentifier) -> bool {
        other.as_str().eq(self.as_str())
    }
}

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

    use ruma_common::{UserId, canonical_json::assert_to_canonical_json_eq, room_id, user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DirectEvent, DirectEventContent};
    use crate::direct::DirectUserIdentifier;

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
                alice.clone(): rooms,
                alice_mail: mail_rooms,
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value(json_data).unwrap();

        let direct_rooms = event.content.get(&DirectUserIdentifier::from(alice)).unwrap();
        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));

        let email_direct_rooms =
            event.content.get(&DirectUserIdentifier::from(alice_mail)).unwrap();
        assert!(email_direct_rooms.contains(&mail_rooms[0]));
    }

    #[test]
    fn user_id_conversion() {
        let alice_direct_uid = DirectUserIdentifier::from("@alice:ruma.io");
        let alice_user_id: UserId = alice_direct_uid
            .clone()
            .try_into()
            .expect("@alice:ruma.io should be convertible into a Matrix user ID");
        assert_eq!(alice_direct_uid, alice_user_id);

        let alice_direct_uid_mail = DirectUserIdentifier::from("alice@ruma.io");
        UserId::try_from(alice_direct_uid_mail)
            .expect_err("alice@ruma.io should not be convertible into a Matrix user ID");

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_direct_uid_mail: DirectUserIdentifier = alice_user_id.clone().into();
        assert_eq!(alice_direct_uid_mail, alice_user_id);
        assert_eq!(alice_user_id, alice_direct_uid_mail);

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_user_id_json = to_json_value(&alice_user_id).unwrap();
        let alice_direct_uid_mail: DirectUserIdentifier =
            from_json_value(alice_user_id_json).unwrap();
        assert_eq!(alice_user_id, alice_direct_uid_mail);
    }
}
