//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use ruma_macros::IdZst;

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
        self.as_bytes()[0] == b'!'
    }

    /// Whether this is a room alias id (starts with `'#'`)
    pub fn is_room_alias_id(&self) -> bool {
        self.as_bytes()[0] == b'#'
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
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
            serde_json::from_str::<OwnedRoomOrAliasId>(r##""!29fhd83h92h0:example.com""##)
                .expect("Failed to convert JSON to RoomId"),
            <&RoomOrAliasId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
