use ruma_macros::IdDst;

use super::{IdParseError, OwnedUserId, UserId};

/// An user identifier, it can be a [`UserId`] or a third-party identifier
/// like an email or a phone number.
///
/// There is no validation on this type, any string is allowed,
/// but you can use `as_user_id` or `into_user_id` to try to get an [`UserId`].
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
#[ruma_id(smallvec_inline_bytes = 40)]
pub struct DirectUserIdentifier(str);

impl DirectUserIdentifier {
    /// Get this `DirectUserIdentifier` as an [`UserId`] if it is one.
    pub fn as_user_id(&self) -> Option<&UserId> {
        self.as_str().try_into().ok()
    }
}

impl OwnedDirectUserIdentifier {
    /// Get this `OwnedDirectUserIdentifier` as an [`UserId`] if it is one.
    pub fn as_user_id(&self) -> Option<&UserId> {
        self.as_str().try_into().ok()
    }

    /// Get this `OwnedDirectUserIdentifier` as an [`OwnedUserId`] if it is one.
    pub fn into_user_id(self) -> Option<OwnedUserId> {
        self.try_into().ok()
    }
}

impl TryFrom<OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        ruma_identifiers_validation::user_id::validate(value.as_str())?;
        Ok(unsafe { Self::from_inner_unchecked(value.into_inner()) })
    }
}

impl TryFrom<&OwnedDirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &OwnedDirectUserIdentifier) -> Result<Self, Self::Error> {
        ruma_identifiers_validation::user_id::validate(value.as_str())?;
        Ok(unsafe { Self::from_inner_unchecked(value.clone().into_inner()) })
    }
}

impl TryFrom<&DirectUserIdentifier> for OwnedUserId {
    type Error = IdParseError;

    fn try_from(value: &DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl<'a> TryFrom<&'a DirectUserIdentifier> for &'a UserId {
    type Error = IdParseError;

    fn try_from(value: &'a DirectUserIdentifier) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl From<OwnedUserId> for OwnedDirectUserIdentifier {
    fn from(value: OwnedUserId) -> Self {
        unsafe { Self::from_inner_unchecked(value.into_inner()) }
    }
}

impl From<&OwnedUserId> for OwnedDirectUserIdentifier {
    fn from(value: &OwnedUserId) -> Self {
        unsafe { Self::from_inner_unchecked(value.clone().into_inner()) }
    }
}

impl From<&UserId> for OwnedDirectUserIdentifier {
    fn from(value: &UserId) -> Self {
        Self::from_str_unchecked(value.as_str())
    }
}

impl<'a> From<&'a UserId> for &'a DirectUserIdentifier {
    fn from(value: &'a UserId) -> Self {
        DirectUserIdentifier::from_borrowed_unchecked(value.as_str())
    }
}

impl PartialEq<&UserId> for &DirectUserIdentifier {
    fn eq(&self, other: &&UserId) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<&DirectUserIdentifier> for &UserId {
    fn eq(&self, other: &&DirectUserIdentifier) -> bool {
        other.as_str().eq(self.as_str())
    }
}

impl PartialEq<OwnedUserId> for &DirectUserIdentifier {
    fn eq(&self, other: &OwnedUserId) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<&DirectUserIdentifier> for OwnedUserId {
    fn eq(&self, other: &&DirectUserIdentifier) -> bool {
        other.as_str().eq(self.as_str())
    }
}

impl PartialEq<&UserId> for OwnedDirectUserIdentifier {
    fn eq(&self, other: &&UserId) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<OwnedDirectUserIdentifier> for &UserId {
    fn eq(&self, other: &OwnedDirectUserIdentifier) -> bool {
        other.as_str().eq(self.as_str())
    }
}

impl PartialEq<OwnedUserId> for OwnedDirectUserIdentifier {
    fn eq(&self, other: &OwnedUserId) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<OwnedDirectUserIdentifier> for OwnedUserId {
    fn eq(&self, other: &OwnedDirectUserIdentifier) -> bool {
        other.as_str().eq(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, to_value as to_json_value};

    use super::{DirectUserIdentifier, OwnedDirectUserIdentifier};
    use crate::{OwnedUserId, user_id};

    #[test]
    fn user_id_conversion() {
        let alice_direct_uid = <&DirectUserIdentifier>::from("@alice:ruma.io");
        let alice_owned_user_id: OwnedUserId = alice_direct_uid
            .to_owned()
            .try_into()
            .expect("@alice:ruma.io should be convertible into a Matrix user ID");
        assert_eq!(alice_direct_uid, alice_owned_user_id);

        let alice_direct_uid_mail = <&DirectUserIdentifier>::from("alice@ruma.io");
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

        let alice_user_id = user_id!("@alice:ruma.io");
        let alice_user_id_json = to_json_value(alice_user_id).unwrap();
        let alice_direct_uid_mail: OwnedDirectUserIdentifier =
            from_json_value(alice_user_id_json).unwrap();
        assert_eq!(alice_user_id, alice_direct_uid_mail);
    }
}
