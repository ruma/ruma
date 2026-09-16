//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::hint::unreachable_unchecked;

use ruma_macros::IdDst;

use super::{OwnedRoomId, RoomAliasId, RoomId, server_name::ServerName};

/// A Matrix [room ID] or a Matrix [room alias ID].
///
/// `RoomOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # use ruma_common::RoomOrAliasId;
/// assert_eq!(<&RoomOrAliasId>::try_from("#ruma:example.com").unwrap(), "#ruma:example.com");
///
/// assert_eq!(
///     <&RoomOrAliasId>::try_from("!n8f893n9:example.com").unwrap(),
///     "!n8f893n9:example.com"
/// );
/// ```
///
/// It can be converted to a `RoomId` or a `RoomAliasId` using `::try_from()` / `.try_into()`.
/// For example, `<&RoomId>::try_from(room_or_alias_id)` returns either `Ok(room_id)` or
/// `Err(room_alias_id)`.
///
/// [room ID]: https://spec.matrix.org/v1.19/appendices/#room-ids
/// [room alias ID]: https://spec.matrix.org/v1.19/appendices/#room-aliases
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
#[ruma_id(validate = ruma_identifiers_validation::room_id_or_alias_id::validate, smallvec_inline_bytes = 48)]
pub struct RoomOrAliasId(str);

impl RoomOrAliasId {
    /// Returns the server name of the room (alias) ID.
    pub fn server_name(&self) -> Option<&ServerName> {
        // We can use the room ID function because the server name in a room alias is already
        // validated.
        super::room_id::find_server_name(self.as_str())
    }

    /// Whether this is a room id (starts with `'!'`)
    pub fn is_room_id(&self) -> bool {
        self.variant() == Variant::RoomId
    }

    /// Whether this is a room alias id (starts with `'#'`)
    pub fn is_room_alias_id(&self) -> bool {
        self.variant() == Variant::RoomAliasId
    }

    fn variant(&self) -> Variant {
        match self.as_bytes().first() {
            Some(b'!') => Variant::RoomId,
            Some(b'#') => Variant::RoomAliasId,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[derive(PartialEq, Eq)]
enum Variant {
    RoomId,
    RoomAliasId,
}

impl<'a> From<&'a RoomId> for &'a RoomOrAliasId {
    fn from(room_id: &'a RoomId) -> Self {
        RoomOrAliasId::from_borrowed_unchecked(room_id.as_str())
    }
}

impl<'a> From<&'a RoomAliasId> for &'a RoomOrAliasId {
    fn from(room_alias_id: &'a RoomAliasId) -> Self {
        RoomOrAliasId::from_borrowed_unchecked(room_alias_id.as_str())
    }
}

impl From<OwnedRoomId> for OwnedRoomOrAliasId {
    fn from(room_id: OwnedRoomId) -> Self {
        unsafe { Self::from_inner_unchecked(room_id.into_inner()) }
    }
}

impl From<RoomAliasId> for OwnedRoomOrAliasId {
    fn from(room_alias_id: RoomAliasId) -> Self {
        unsafe { Self::from_inner_unchecked(room_alias_id.into_inner()) }
    }
}

impl TryFrom<OwnedRoomOrAliasId> for OwnedRoomId {
    type Error = RoomAliasId;

    fn try_from(id: OwnedRoomOrAliasId) -> Result<OwnedRoomId, RoomAliasId> {
        let variant = id.variant();
        let inner = id.into_inner();

        unsafe {
            match variant {
                Variant::RoomId => Ok(Self::from_inner_unchecked(inner)),
                Variant::RoomAliasId => Err(RoomAliasId::from_inner_unchecked(inner)),
            }
        }
    }
}

impl TryFrom<OwnedRoomOrAliasId> for RoomAliasId {
    type Error = OwnedRoomId;

    fn try_from(id: OwnedRoomOrAliasId) -> Result<RoomAliasId, OwnedRoomId> {
        let variant = id.variant();
        let inner = id.into_inner();

        unsafe {
            match variant {
                Variant::RoomAliasId => Ok(Self::from_inner_unchecked(inner)),
                Variant::RoomId => Err(OwnedRoomId::from_inner_unchecked(inner)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnedRoomOrAliasId, RoomOrAliasId};
    use crate::IdParseError;

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            <&RoomOrAliasId>::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .as_str(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_str(),
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
            serde_json::from_str::<OwnedRoomOrAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            "#ruma:example.com"
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::from_str::<OwnedRoomOrAliasId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            "!29fhd83h92h0:example.com"
        );
    }
}
