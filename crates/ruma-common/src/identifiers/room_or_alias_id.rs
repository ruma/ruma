//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::hint::unreachable_unchecked;

use ruma_macros::ruma_id;
use tracing::warn;

use super::{RoomAliasId, RoomId, ServerName};

/// A Matrix [room ID] or a Matrix [room alias ID].
///
/// `RoomOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # use ruma_common::RoomOrAliasId;
/// assert_eq!(RoomOrAliasId::try_from("#ruma:example.com").unwrap(), "#ruma:example.com");
///
/// assert_eq!(RoomOrAliasId::try_from("!n8f893n9:example.com").unwrap(), "!n8f893n9:example.com");
/// ```
///
/// It can be converted to a `RoomId` or a `RoomAliasId` using `::try_from()` / `.try_into()`.
/// For example, `RoomId::try_from(room_or_alias_id)` returns either `Ok(room_id)` or
/// `Err(room_alias_id)`.
///
/// [room ID]: https://spec.matrix.org/latest/appendices/#room-ids
/// [room alias ID]: https://spec.matrix.org/latest/appendices/#room-aliases
#[ruma_id(validate = ruma_identifiers_validation::room_id_or_alias_id::validate)]
pub struct RoomOrAliasId;

impl RoomOrAliasId {
    /// Returns the server name of the room (alias) ID.
    pub fn server_name(&self) -> Option<ServerName> {
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

impl From<&RoomId> for RoomOrAliasId {
    fn from(id: &RoomId) -> Self {
        id.to_owned().into()
    }
}

impl From<&RoomAliasId> for RoomOrAliasId {
    fn from(id: &RoomAliasId) -> Self {
        id.to_owned().into()
    }
}

impl From<RoomId> for RoomOrAliasId {
    fn from(id: RoomId) -> Self {
        unsafe { Self::from_inner_unchecked(id.into_inner()) }
    }
}

impl From<RoomAliasId> for RoomOrAliasId {
    fn from(id: RoomAliasId) -> Self {
        unsafe { Self::from_inner_unchecked(id.into_inner()) }
    }
}

impl TryFrom<RoomOrAliasId> for RoomId {
    type Error = RoomAliasId;

    fn try_from(id: RoomOrAliasId) -> Result<RoomId, RoomAliasId> {
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

impl TryFrom<RoomOrAliasId> for RoomAliasId {
    type Error = RoomId;

    fn try_from(id: RoomOrAliasId) -> Result<RoomAliasId, RoomId> {
        let variant = id.variant();
        let inner = id.into_inner();

        unsafe {
            match variant {
                Variant::RoomAliasId => Ok(Self::from_inner_unchecked(inner)),
                Variant::RoomId => Err(RoomId::from_inner_unchecked(inner)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoomOrAliasId;
    use crate::IdParseError;

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            RoomOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId."),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            RoomOrAliasId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId."),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn missing_sigil_for_room_id_or_alias_id() {
        assert_eq!(
            RoomOrAliasId::try_from("ruma:example.com").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::to_string(
                &RoomOrAliasId::try_from("#ruma:example.com")
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
                &RoomOrAliasId::try_from("!29fhd83h92h0:example.com")
                    .expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::from_str::<RoomOrAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            RoomOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::from_str::<RoomOrAliasId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            RoomOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
