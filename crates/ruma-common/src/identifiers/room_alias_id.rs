//! Matrix room alias identifiers.

use super::{matrix_uri::UriAction, server_name::ServerName, EventId, MatrixToUri, MatrixUri};

/// A Matrix [room alias ID].
///
/// A `RoomAliasId` is converted from a string slice, and can be converted back into a string as
/// needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_common::RoomAliasId;
/// assert_eq!(<&RoomAliasId>::try_from("#ruma:example.com").unwrap(), "#ruma:example.com");
/// ```
///
/// [room alias ID]: https://spec.matrix.org/v1.2/appendices/#room-aliases
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomAliasId(str);

opaque_identifier_validated!(RoomAliasId, ruma_identifiers_validation::room_alias_id::validate);

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
    pub fn matrix_to_event_uri(&self, ev_id: &EventId) -> MatrixToUri {
        MatrixToUri::new((self, ev_id).into(), Vec::new())
    }

    /// Create a `matrix:` URI for this room alias ID.
    ///
    /// If `join` is `true`, a click on the URI should join the room.
    pub fn matrix_uri(&self, join: bool) -> MatrixUri {
        MatrixUri::new(self.into(), Vec::new(), Some(UriAction::Join).filter(|_| join))
    }

    /// Create a `matrix:` URI for an event scoped under this room alias ID.
    pub fn matrix_event_uri(&self, ev_id: &EventId) -> MatrixUri {
        MatrixUri::new((self, ev_id).into(), Vec::new(), None)
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::RoomAliasId;
    use crate::Error;

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
            serde_json::from_str::<Box<RoomAliasId>>(r##""#ruma:example.com""##)
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
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_room_alias_id_delimiter() {
        assert_eq!(<&RoomAliasId>::try_from("#ruma").unwrap_err(), Error::MissingDelimiter);
    }

    #[test]
    fn invalid_leading_sigil() {
        assert_eq!(
            <&RoomAliasId>::try_from("!room_id:foo.bar").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn invalid_room_alias_id_host() {
        assert_eq!(<&RoomAliasId>::try_from("#ruma:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_room_alias_id_port() {
        assert_eq!(
            <&RoomAliasId>::try_from("#ruma:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
