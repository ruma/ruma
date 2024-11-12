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

/// An user identifier, it can be a [`UserId`] or a third-party identifier
/// like an email or a phone number.
///
/// There is no validation on this type, any string is allowed,
/// but you can use `as_user_id` or `into_user_id` to try to get an [`UserId`].
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
pub struct DirectUserIdentifier(str);

impl DirectUserIdentifier {
    /// Get this `DirectUserIdentifier` as an [`UserId`] if it is one.
    pub fn as_user_id(&self) -> Option<&UserId> {
        self.0.try_into().ok()
    }
}

impl OwnedDirectUserIdentifier {
    /// Get this `OwnedDirectUserIdentifier` as an [`UserId`] if it is one.
    pub fn as_user_id(&self) -> Option<&UserId> {
        self.0.try_into().ok()
    }

    /// Get this `OwnedDirectUserIdentifier` as an [`OwnedUserId`] if it is one.
    pub fn into_user_id(self) -> Option<OwnedUserId> {
        OwnedUserId::try_from(self).ok()
    }
}

impl TryFrom<OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl TryFrom<&OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl TryFrom<&DirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl<'a> TryFrom<&'a DirectUserIdentifier> for &'a UserId {
    type Error = IdParseError;

    fn try_from(value: &'a DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.0.try_into()
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

impl PartialEq<&UserId> for &DirectUserIdentifier {
    fn eq(&self, other: &&UserId) -> bool {
        self.0.eq(other.as_str())
    }
}

impl PartialEq<&DirectUserIdentifier> for &UserId {
    fn eq(&self, other: &&DirectUserIdentifier) -> bool {
        other.0.eq(self.as_str())
    }
}

impl PartialEq<OwnedUserId> for &DirectUserIdentifier {
    fn eq(&self, other: &OwnedUserId) -> bool {
        self.0.eq(other.as_str())
    }
}

impl PartialEq<&DirectUserIdentifier> for OwnedUserId {
    fn eq(&self, other: &&DirectUserIdentifier) -> bool {
        other.0.eq(self.as_str())
    }
}

impl PartialEq<&UserId> for OwnedDirectUserIdentifier {
    fn eq(&self, other: &&UserId) -> bool {
        self.0.eq(other.as_str())
    }
}

impl PartialEq<OwnedDirectUserIdentifier> for &UserId {
    fn eq(&self, other: &OwnedDirectUserIdentifier) -> bool {
        other.0.eq(self.as_str())
    }
}

impl PartialEq<OwnedUserId> for OwnedDirectUserIdentifier {
    fn eq(&self, other: &OwnedUserId) -> bool {
        self.0.eq(other.as_str())
    }
}

impl PartialEq<OwnedDirectUserIdentifier> for OwnedUserId {
    fn eq(&self, other: &OwnedDirectUserIdentifier) -> bool {
        other.0.eq(self.as_str())
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
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DirectEvent, DirectEventContent};
    use crate::direct::{DirectUserIdentifier, OwnedDirectUserIdentifier};

    #[test]
    fn serialization() {
        let mut content = DirectEventContent(BTreeMap::new());
        let alice = user_id!("@alice:ruma.io");
        let alice_mail = "alice@ruma.io";
        let rooms = vec![owned_room_id!("!1:ruma.io")];
        let mail_rooms = vec![owned_room_id!("!3:ruma.io")];

        content.insert(alice.into(), rooms.clone());
        content.insert(alice_mail.into(), mail_rooms.clone());

        let json_data = json!({
            alice: rooms,
            alice_mail: mail_rooms,
        });

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let alice = user_id!("@alice:ruma.io");
        let alice_mail = "alice@ruma.io";
        let rooms = vec![owned_room_id!("!1:ruma.io"), owned_room_id!("!2:ruma.io")];
        let mail_rooms = vec![owned_room_id!("!3:ruma.io")];

        let json_data = json!({
            "content": {
                alice: rooms,
                alice_mail: mail_rooms,
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value(json_data).unwrap();

        let direct_rooms = event.content.get(<&DirectUserIdentifier>::from(alice)).unwrap();
        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));

        let email_direct_rooms =
            event.content.get(<&DirectUserIdentifier>::from(alice_mail)).unwrap();
        assert!(email_direct_rooms.contains(&mail_rooms[0]));
    }

    #[test]
    fn user_id_conversion() {
        let alice_direct_uid = DirectUserIdentifier::from_borrowed("@alice:ruma.io");
        let alice_owned_user_id: OwnedUserId = alice_direct_uid
            .to_owned()
            .try_into()
            .expect("@alice:ruma.io should be convertible into a Matrix user ID");
        assert_eq!(alice_direct_uid, alice_owned_user_id);

        let alice_direct_uid_mail = DirectUserIdentifier::from_borrowed("alice@ruma.io");
        OwnedUserId::try_from(alice_direct_uid_mail.to_owned())
            .expect_err("alice@ruma.io should not be convertible into a Matrix user ID");

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_direct_uid_mail: &DirectUserIdentifier = alice_user_id.into();
        assert_eq!(alice_direct_uid_mail, alice_user_id);
        assert_eq!(alice_direct_uid_mail, alice_user_id.to_owned());
        assert_eq!(alice_user_id, alice_direct_uid_mail);
        assert_eq!(alice_user_id.to_owned(), alice_direct_uid_mail);

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_direct_uid_mail: OwnedDirectUserIdentifier = alice_user_id.into();
        assert_eq!(alice_direct_uid_mail, alice_user_id);
        assert_eq!(alice_direct_uid_mail, alice_user_id.to_owned());
        assert_eq!(alice_user_id, alice_direct_uid_mail);
        assert_eq!(alice_user_id.to_owned(), alice_direct_uid_mail);
    }
}
