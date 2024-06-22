//! Matrix room alias identifiers.

use ruma_macros::IdZst;

use super::{matrix_uri::UriAction, server_name::ServerName, MatrixToUri, MatrixUri, OwnedEventId};

/// A Matrix [room alias ID].
///
/// A `RoomAliasId` is converted from a string slice, and can be converted back into a string as
/// needed.
///
/// ```
/// # use ruma_common::RoomAliasId;
/// assert_eq!(<&RoomAliasId>::try_from("#ruma:example.com").unwrap(), "#ruma:example.com");
/// ```
///
/// [room alias ID]: https://spec.matrix.org/latest/appendices/#room-aliases
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::room_alias_id::validate)]
pub struct RoomAliasId(str);

impl RoomAliasId {
    /// Returns the room's alias.
    pub fn alias(&self) -> &str {
        &self.as_str()[1..self.colon_idx()]
    }

    /// Returns the server name of the room alias ID.
    pub fn server_name(&self) -> &ServerName {
        ServerName::from_borrowed(&self.as_str()[self.colon_idx() + 1..])
    }

    /// Create a `matrix.to` URI for this room alias ID.
    pub fn matrix_to_uri(&self) -> MatrixToUri {
        MatrixToUri::new(self.into(), Vec::new())
    }

    /// Create a `matrix.to` URI for an event scoped under this room alias ID.
    ///
    /// This is deprecated because room aliases are mutable, so the URI might break after a while.
    #[deprecated = "Use `RoomId::matrix_to_event_uri` instead."]
    pub fn matrix_to_event_uri(&self, ev_id: impl Into<OwnedEventId>) -> MatrixToUri {
        MatrixToUri::new((self.to_owned(), ev_id.into()).into(), Vec::new())
    }

    /// Create a `matrix:` URI for this room alias ID.
    ///
    /// If `join` is `true`, a click on the URI should join the room.
    pub fn matrix_uri(&self, join: bool) -> MatrixUri {
        MatrixUri::new(self.into(), Vec::new(), Some(UriAction::Join).filter(|_| join))
    }

    /// Create a `matrix:` URI for an event scoped under this room alias ID.
    ///
    /// This is deprecated because room aliases are mutable, so the URI might break after a while.
    #[deprecated = "Use `RoomId::matrix_event_uri` instead."]
    pub fn matrix_event_uri(&self, ev_id: impl Into<OwnedEventId>) -> MatrixUri {
        MatrixUri::new((self.to_owned(), ev_id.into()).into(), Vec::new(), None)
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnedRoomAliasId, RoomAliasId};
    use crate::IdParseError;

    #[test]
    fn valid_room_alias_id() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:example.com").expect("Failed to create RoomAliasId."),
            "#ruma:example.com"
        );
    }

    #[test]
    fn empty_localpart() {
        assert_eq!(
            <&RoomAliasId>::try_from("#:myhomeserver.io").expect("Failed to create RoomAliasId."),
            "#:myhomeserver.io"
        );
    }

    #[test]
    fn serialize_valid_room_alias_id() {
        assert_eq!(
            serde_json::to_string(
                <&RoomAliasId>::try_from("#ruma:example.com")
                    .expect("Failed to create RoomAliasId.")
            )
            .expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn deserialize_valid_room_alias_id() {
        assert_eq!(
            serde_json::from_str::<OwnedRoomAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            <&RoomAliasId>::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn valid_room_alias_id_with_explicit_standard_port() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:example.com:443")
                .expect("Failed to create RoomAliasId."),
            "#ruma:example.com:443"
        );
    }

    #[test]
    fn valid_room_alias_id_with_non_standard_port() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:example.com:5000")
                .expect("Failed to create RoomAliasId."),
            "#ruma:example.com:5000"
        );
    }

    #[test]
    fn valid_room_alias_id_unicode() {
        assert_eq!(
            <&RoomAliasId>::try_from("#老虎Â£я:example.com")
                .expect("Failed to create RoomAliasId."),
            "#老虎Â£я:example.com"
        );
    }

    #[test]
    fn missing_room_alias_id_sigil() {
        assert_eq!(
            <&RoomAliasId>::try_from("39hvsi03hlne:example.com").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_room_alias_id_delimiter() {
        assert_eq!(<&RoomAliasId>::try_from("#ruma").unwrap_err(), IdParseError::MissingColon);
    }

    #[test]
    fn invalid_leading_sigil() {
        assert_eq!(
            <&RoomAliasId>::try_from("!room_id:foo.bar").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn invalid_room_alias_id_host() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:/").unwrap_err(),
            IdParseError::InvalidServerName
        );
    }

    #[test]
    fn invalid_room_alias_id_port() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:example.com:notaport").unwrap_err(),
            IdParseError::InvalidServerName
        );
    }
}
