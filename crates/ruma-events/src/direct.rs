//! Types for the [`m.direct`] event.
//!
//! [`m.direct`]: https://spec.matrix.org/latest/client-server-api/#mdirect

use std::{
    collections::{btree_map, BTreeMap},
    ops::{Deref, DerefMut},
};

use ruma_common::{IdParseError, OwnedRoomId, OwnedUserId, UserId};
use ruma_macros::{EventContent, IdZst};
use serde::{Deserialize, Serialize};

/// An user identifier, it can be a MXID or a third-party identifier
/// like an email or a phone number.
///
/// There is no validation on this type, any string is allowed,
/// but you can use `to_user_id` to try to get a MXID.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
pub struct DirectUserIdentifier(str);

impl TryFrom<OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        Self::try_from(&value.0)
    }
}

impl TryFrom<&OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        Self::try_from(&value.0)
    }
}

impl TryFrom<&DirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &DirectUserIdentifier) -> Result<Self, Self::Error> {
        Self::try_from(&value.0)
    }
}

impl From<OwnedUserId> for OwnedDirectUserIdentifier {
    fn from(value: OwnedUserId) -> Self {
        DirectUserIdentifier::from_borrowed(value.as_str()).to_owned()
    }
}

impl From<&OwnedUserId> for OwnedDirectUserIdentifier {
    fn from(value: &OwnedUserId) -> Self {
        DirectUserIdentifier::from_borrowed(value.as_str()).to_owned()
    }
}

impl From<&UserId> for OwnedDirectUserIdentifier {
    fn from(value: &UserId) -> Self {
        DirectUserIdentifier::from_borrowed(value.as_str()).to_owned()
    }
}

impl<'a> From<&'a UserId> for &'a DirectUserIdentifier {
    fn from(value: &'a UserId) -> Self {
        DirectUserIdentifier::from_borrowed(value.as_str())
    }
}

/// The content of an `m.direct` event.
///
/// A mapping of `DirectUserIdentifier`s to a list of `RoomId`s which are considered *direct* for
/// that particular user.
///
/// Informs the client about the rooms that are considered direct by a user.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.direct", kind = GlobalAccountData)]
pub struct DirectEventContent(pub BTreeMap<OwnedDirectUserIdentifier, Vec<OwnedRoomId>>);

impl Deref for DirectEventContent {
    type Target = BTreeMap<OwnedDirectUserIdentifier, Vec<OwnedRoomId>>;

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
    type Item = (OwnedDirectUserIdentifier, Vec<OwnedRoomId>);
    type IntoIter = btree_map::IntoIter<OwnedDirectUserIdentifier, Vec<OwnedRoomId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(OwnedDirectUserIdentifier, Vec<OwnedRoomId>)> for DirectEventContent {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (OwnedDirectUserIdentifier, Vec<OwnedRoomId>)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_common::{owned_room_id, user_id, OwnedUserId};
    // use ruma_macros::user_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DirectEvent, DirectEventContent};
    use crate::direct::{DirectUserIdentifier, OwnedDirectUserIdentifier};

    #[test]
    fn serialization() {
        let mut content = DirectEventContent(BTreeMap::new());
        let alice = DirectUserIdentifier::from_borrowed("@alice:ruma.io");
        let rooms = vec![owned_room_id!("!1:ruma.io")];

        content.insert(alice.to_owned(), rooms.clone());

        let json_data = json!({
            alice: rooms,
        });

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let alice = DirectUserIdentifier::from_borrowed("@alice:ruma.io");
        let rooms = vec![owned_room_id!("!1:ruma.io"), owned_room_id!("!2:ruma.io")];

        let json_data = json!({
            "content": {
                alice: rooms,
                "alice@ruma.io": vec![owned_room_id!("!3:ruma.io")],
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value(json_data).unwrap();
        let direct_rooms = event.content.get(alice).unwrap();

        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));
    }

    #[test]
    fn user_id_conversion() {
        let alice_direct_uid = DirectUserIdentifier::from_borrowed("@alice:ruma.io");
        let alice_owned_user_id: OwnedUserId = alice_direct_uid
            .to_owned()
            .try_into()
            .expect("@alice:ruma.io should be convertible into a Matrix user ID");
        assert_eq!(alice_direct_uid.as_str(), alice_owned_user_id.as_str());

        let alice_direct_uid_mail = DirectUserIdentifier::from_borrowed("alice@ruma.io");
        OwnedUserId::try_from(alice_direct_uid_mail.to_owned())
            .expect_err("alice@ruma.io should not be convertible into a Matrix user ID");

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_direct_uid_mail: &DirectUserIdentifier = alice_user_id.into();
        assert_eq!(alice_direct_uid_mail.as_str(), alice_user_id.as_str());

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_direct_uid_mail: OwnedDirectUserIdentifier = alice_user_id.into();
        assert_eq!(alice_direct_uid_mail.as_str(), alice_user_id.as_str());
    }
}
