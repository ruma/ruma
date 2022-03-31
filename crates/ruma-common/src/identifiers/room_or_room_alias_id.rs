//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::{convert::TryFrom, hint::unreachable_unchecked};

use super::{server_name::ServerName, RoomAliasId, RoomId};

/// A Matrix [room ID] or a Matrix [room alias ID].
///
/// `RoomOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_common::RoomOrAliasId;
/// assert_eq!(<&RoomOrAliasId>::try_from("#ruma:example.com").unwrap(), "#ruma:example.com");
///
/// assert_eq!(
///     <&RoomOrAliasId>::try_from("!n8f893n9:example.com").unwrap(),
///     "!n8f893n9:example.com"
/// );
/// ```
///
/// [room ID]: https://spec.matrix.org/v1.2/appendices/#room-ids-and-event-ids
/// [room alias ID]: https://spec.matrix.org/v1.2/appendices/#room-aliases
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomOrAliasId(str);

owned_identifier!(OwnedRoomOrAliasId, RoomOrAliasId);

opaque_identifier_validated!(
    RoomOrAliasId,
    OwnedRoomOrAliasId,
    ruma_identifiers_validation::room_id_or_alias_id::validate
);

impl RoomOrAliasId {
    /// Returns the local part (everything after the `!` or `#` and before the first colon).
    pub fn localpart(&self) -> &str {
        &self.as_str()[1..self.colon_idx()]
    }

    /// Returns the server name of the room (alias) ID.
    pub fn server_name(&self) -> &ServerName {
        ServerName::from_borrowed(&self.as_str()[self.colon_idx() + 1..])
    }

    /// Whether this is a room id (starts with `'!'`)
    pub fn is_room_id(&self) -> bool {
        self.variant() == Variant::RoomId
    }

    /// Whether this is a room alias id (starts with `'#'`)
    pub fn is_room_alias_id(&self) -> bool {
        self.variant() == Variant::RoomAliasId
    }

    /// Turn this `RoomOrAliasId` into `Either<RoomId, RoomAliasId>`
    #[cfg(feature = "either")]
    pub fn into_either(self: Box<Self>) -> either::Either<Box<RoomId>, Box<RoomAliasId>> {
        let variant = self.variant();
        let boxed_str = self.into_owned();

        match variant {
            Variant::RoomId => either::Either::Left(RoomId::from_owned(boxed_str)),
            Variant::RoomAliasId => either::Either::Right(RoomAliasId::from_owned(boxed_str)),
        }
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }

    fn variant(&self) -> Variant {
        match self.as_str().bytes().next() {
            Some(b'!') => Variant::RoomId,
            Some(b'#') => Variant::RoomAliasId,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[derive(PartialEq)]
enum Variant {
    RoomId,
    RoomAliasId,
}

impl<'a> From<&'a RoomId> for &'a RoomOrAliasId {
    fn from(room_id: &'a RoomId) -> Self {
        RoomOrAliasId::from_borrowed(room_id.as_str())
    }
}

impl<'a> From<&'a RoomAliasId> for &'a RoomOrAliasId {
    fn from(room_alias_id: &'a RoomAliasId) -> Self {
        RoomOrAliasId::from_borrowed(room_alias_id.as_str())
    }
}

impl From<Box<RoomId>> for Box<RoomOrAliasId> {
    fn from(room_id: Box<RoomId>) -> Self {
        RoomOrAliasId::from_owned(room_id.into_owned())
    }
}

impl From<Box<RoomAliasId>> for Box<RoomOrAliasId> {
    fn from(room_alias_id: Box<RoomAliasId>) -> Self {
        RoomOrAliasId::from_owned(room_alias_id.into_owned())
    }
}

impl<'a> TryFrom<&'a RoomOrAliasId> for &'a RoomId {
    type Error = &'a RoomAliasId;

    fn try_from(id: &'a RoomOrAliasId) -> Result<&'a RoomId, &'a RoomAliasId> {
        match id.variant() {
            Variant::RoomId => Ok(RoomId::from_borrowed(id.as_str())),
            Variant::RoomAliasId => Err(RoomAliasId::from_borrowed(id.as_str())),
        }
    }
}

impl<'a> TryFrom<&'a RoomOrAliasId> for &'a RoomAliasId {
    type Error = &'a RoomId;

    fn try_from(id: &'a RoomOrAliasId) -> Result<&'a RoomAliasId, &'a RoomId> {
        match id.variant() {
            Variant::RoomAliasId => Ok(RoomAliasId::from_borrowed(id.as_str())),
            Variant::RoomId => Err(RoomId::from_borrowed(id.as_str())),
        }
    }
}

impl TryFrom<Box<RoomOrAliasId>> for Box<RoomId> {
    type Error = Box<RoomAliasId>;

    fn try_from(id: Box<RoomOrAliasId>) -> Result<Box<RoomId>, Box<RoomAliasId>> {
        match id.variant() {
            Variant::RoomId => Ok(RoomId::from_owned(id.into_owned())),
            Variant::RoomAliasId => Err(RoomAliasId::from_owned(id.into_owned())),
        }
    }
}

impl TryFrom<Box<RoomOrAliasId>> for Box<RoomAliasId> {
    type Error = Box<RoomId>;

    fn try_from(id: Box<RoomOrAliasId>) -> Result<Box<RoomAliasId>, Box<RoomId>> {
        match id.variant() {
            Variant::RoomAliasId => Ok(RoomAliasId::from_owned(id.into_owned())),
            Variant::RoomId => Err(RoomId::from_owned(id.into_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::RoomOrAliasId;
    use crate::IdParseError;

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            <&RoomOrAliasId>::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .as_ref(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn missing_sigil_for_room_id_or_alias_id() {
        assert_eq!(
            <&RoomOrAliasId>::try_from("ruma:example.com").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::to_string(
                <&RoomOrAliasId>::try_from("#ruma:example.com")
                    .expect("Failed to create RoomAliasId.")
            )
            .expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::to_string(
                <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                    .expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::from_str::<Box<RoomOrAliasId>>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            <&RoomOrAliasId>::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::from_str::<Box<RoomOrAliasId>>(r##""!29fhd83h92h0:example.com""##)
                .expect("Failed to convert JSON to RoomId"),
            <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
