//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::hint::unreachable_unchecked;

use ruma_macros::IdZst;
use tracing::warn;

use super::{server_name::ServerName, OwnedRoomAliasId, OwnedRoomId, RoomAliasId, RoomId};

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
/// [room ID]: https://spec.matrix.org/latest/appendices/#room-ids
/// [room alias ID]: https://spec.matrix.org/latest/appendices/#room-aliases
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::room_id_or_alias_id::validate)]
pub struct RoomOrAliasId(str);

impl RoomOrAliasId {
    /// Returns the server name of the room (alias) ID.
    pub fn server_name(&self) -> Option<&ServerName> {
        let colon_idx = self.as_str().find(':')?;
        let server_name = &self.as_str()[colon_idx + 1..];
        match server_name.try_into() {
            Ok(parsed) => Some(parsed),
            // Room aliases are verified to contain a server name at parse time
            Err(e) => {
                warn!(
                    target: "ruma_common::identifiers::room_id",
                    server_name,
                    "Room ID contains colon but no valid server name afterwards: {e}",
                );
                None
            }
        }
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
        RoomOrAliasId::from_borrowed(room_id.as_str())
    }
}

impl<'a> From<&'a RoomAliasId> for &'a RoomOrAliasId {
    fn from(room_alias_id: &'a RoomAliasId) -> Self {
        RoomOrAliasId::from_borrowed(room_alias_id.as_str())
    }
}

impl From<OwnedRoomId> for OwnedRoomOrAliasId {
    fn from(room_id: OwnedRoomId) -> Self {
        // FIXME: Don't allocate
        RoomOrAliasId::from_borrowed(room_id.as_str()).to_owned()
    }
}

impl From<OwnedRoomAliasId> for OwnedRoomOrAliasId {
    fn from(room_alias_id: OwnedRoomAliasId) -> Self {
        // FIXME: Don't allocate
        RoomOrAliasId::from_borrowed(room_alias_id.as_str()).to_owned()
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

impl TryFrom<OwnedRoomOrAliasId> for OwnedRoomId {
    type Error = OwnedRoomAliasId;

    fn try_from(id: OwnedRoomOrAliasId) -> Result<OwnedRoomId, OwnedRoomAliasId> {
        // FIXME: Don't allocate
        match id.variant() {
            Variant::RoomId => Ok(RoomId::from_borrowed(id.as_str()).to_owned()),
            Variant::RoomAliasId => Err(RoomAliasId::from_borrowed(id.as_str()).to_owned()),
        }
    }
}

impl TryFrom<OwnedRoomOrAliasId> for OwnedRoomAliasId {
    type Error = OwnedRoomId;

    fn try_from(id: OwnedRoomOrAliasId) -> Result<OwnedRoomAliasId, OwnedRoomId> {
        // FIXME: Don't allocate
        match id.variant() {
            Variant::RoomAliasId => Ok(RoomAliasId::from_borrowed(id.as_str()).to_owned()),
            Variant::RoomId => Err(RoomId::from_borrowed(id.as_str()).to_owned()),
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
            <&RoomOrAliasId>::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::from_str::<OwnedRoomOrAliasId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
