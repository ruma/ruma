//! Matrix room identifiers.

use ruma_macros::IdZst;

use super::{
    matrix_uri::UriAction, MatrixToUri, MatrixUri, OwnedEventId, OwnedServerName, ServerName,
};
use crate::RoomOrAliasId;

/// A Matrix [room ID].
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use ruma_common::RoomId;
/// assert_eq!(<&RoomId>::try_from("!n8f893n9:example.com").unwrap(), "!n8f893n9:example.com");
/// ```
///
/// [room ID]: https://spec.matrix.org/latest/appendices/#room-ids
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::room_id::validate)]
pub struct RoomId(str);

impl RoomId {
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    #[cfg(feature = "rand")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(server_name: &ServerName) -> OwnedRoomId {
        Self::from_borrowed(&format!("!{}:{server_name}", super::generate_localpart(18))).to_owned()
    }

    /// Returns the server name of the room ID.
    pub fn server_name(&self) -> Option<&ServerName> {
        <&RoomOrAliasId>::from(self).server_name()
    }

    /// Create a `matrix.to` URI for this room ID.
    ///
    /// Note that it is recommended to provide servers that should know the room to be able to find
    /// it with its room ID. For that use [`RoomId::matrix_to_uri_via()`].
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_common::{room_id, server_name};
    ///
    /// assert_eq!(
    ///     room_id!("!somewhere:example.org").matrix_to_uri().to_string(),
    ///     "https://matrix.to/#/!somewhere:example.org"
    /// );
    /// ```
    pub fn matrix_to_uri(&self) -> MatrixToUri {
        MatrixToUri::new(self.into(), vec![])
    }

    /// Create a `matrix.to` URI for this room ID with a list of servers that should know it.
    ///
    /// To get the list of servers, it is recommended to use the [routing algorithm] from the spec.
    ///
    /// If you don't have a list of servers, you can use [`RoomId::matrix_to_uri()`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_common::{room_id, server_name};
    ///
    /// assert_eq!(
    ///     room_id!("!somewhere:example.org")
    ///         .matrix_to_uri_via([&*server_name!("example.org"), &*server_name!("alt.example.org")])
    ///         .to_string(),
    ///     "https://matrix.to/#/!somewhere:example.org?via=example.org&via=alt.example.org"
    /// );
    /// ```
    ///
    /// [routing algorithm]: https://spec.matrix.org/latest/appendices/#routing
    pub fn matrix_to_uri_via<T>(&self, via: T) -> MatrixToUri
    where
        T: IntoIterator,
        T::Item: Into<OwnedServerName>,
    {
        MatrixToUri::new(self.into(), via.into_iter().map(Into::into).collect())
    }

    /// Create a `matrix.to` URI for an event scoped under this room ID.
    ///
    /// Note that it is recommended to provide servers that should know the room to be able to find
    /// it with its room ID. For that use [`RoomId::matrix_to_event_uri_via()`].
    pub fn matrix_to_event_uri(&self, ev_id: impl Into<OwnedEventId>) -> MatrixToUri {
        MatrixToUri::new((self.to_owned(), ev_id.into()).into(), vec![])
    }

    /// Create a `matrix.to` URI for an event scoped under this room ID with a list of servers that
    /// should know it.
    ///
    /// To get the list of servers, it is recommended to use the [routing algorithm] from the spec.
    ///
    /// If you don't have a list of servers, you can use [`RoomId::matrix_to_event_uri()`] instead.
    ///
    /// [routing algorithm]: https://spec.matrix.org/latest/appendices/#routing
    pub fn matrix_to_event_uri_via<T>(&self, ev_id: impl Into<OwnedEventId>, via: T) -> MatrixToUri
    where
        T: IntoIterator,
        T::Item: Into<OwnedServerName>,
    {
        MatrixToUri::new(
            (self.to_owned(), ev_id.into()).into(),
            via.into_iter().map(Into::into).collect(),
        )
    }

    /// Create a `matrix:` URI for this room ID.
    ///
    /// If `join` is `true`, a click on the URI should join the room.
    ///
    /// Note that it is recommended to provide servers that should know the room to be able to find
    /// it with its room ID. For that use [`RoomId::matrix_uri_via()`].
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_common::{room_id, server_name};
    ///
    /// assert_eq!(
    ///     room_id!("!somewhere:example.org").matrix_uri(false).to_string(),
    ///     "matrix:roomid/somewhere:example.org"
    /// );
    /// ```
    pub fn matrix_uri(&self, join: bool) -> MatrixUri {
        MatrixUri::new(self.into(), vec![], Some(UriAction::Join).filter(|_| join))
    }

    /// Create a `matrix:` URI for this room ID with a list of servers that should know it.
    ///
    /// To get the list of servers, it is recommended to use the [routing algorithm] from the spec.
    ///
    /// If you don't have a list of servers, you can use [`RoomId::matrix_uri()`] instead.
    ///
    /// If `join` is `true`, a click on the URI should join the room.
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_common::{room_id, server_name};
    ///
    /// assert_eq!(
    ///     room_id!("!somewhere:example.org")
    ///         .matrix_uri_via(
    ///             [&*server_name!("example.org"), &*server_name!("alt.example.org")],
    ///             true
    ///         )
    ///         .to_string(),
    ///     "matrix:roomid/somewhere:example.org?via=example.org&via=alt.example.org&action=join"
    /// );
    /// ```
    ///
    /// [routing algorithm]: https://spec.matrix.org/latest/appendices/#routing
    pub fn matrix_uri_via<T>(&self, via: T, join: bool) -> MatrixUri
    where
        T: IntoIterator,
        T::Item: Into<OwnedServerName>,
    {
        MatrixUri::new(
            self.into(),
            via.into_iter().map(Into::into).collect(),
            Some(UriAction::Join).filter(|_| join),
        )
    }

    /// Create a `matrix:` URI for an event scoped under this room ID.
    ///
    /// Note that it is recommended to provide servers that should know the room to be able to find
    /// it with its room ID. For that use [`RoomId::matrix_event_uri_via()`].
    pub fn matrix_event_uri(&self, ev_id: impl Into<OwnedEventId>) -> MatrixUri {
        MatrixUri::new((self.to_owned(), ev_id.into()).into(), vec![], None)
    }

    /// Create a `matrix:` URI for an event scoped under this room ID with a list of servers that
    /// should know it.
    ///
    /// To get the list of servers, it is recommended to use the [routing algorithm] from the spec.
    ///
    /// If you don't have a list of servers, you can use [`RoomId::matrix_event_uri()`] instead.
    ///
    /// [routing algorithm]: https://spec.matrix.org/latest/appendices/#routing
    pub fn matrix_event_uri_via<T>(&self, ev_id: impl Into<OwnedEventId>, via: T) -> MatrixUri
    where
        T: IntoIterator,
        T::Item: Into<OwnedServerName>,
    {
        MatrixUri::new(
            (self.to_owned(), ev_id.into()).into(),
            via.into_iter().map(Into::into).collect(),
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnedRoomId, RoomId};
    use crate::{server_name, IdParseError};

    #[test]
    fn valid_room_id() {
        let room_id =
            <&RoomId>::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.");
        assert_eq!(room_id, "!29fhd83h92h0:example.com");
    }

    #[test]
    fn empty_localpart() {
        let room_id = <&RoomId>::try_from("!:example.com").expect("Failed to create RoomId.");
        assert_eq!(room_id, "!:example.com");
        assert_eq!(room_id.server_name(), Some(server_name!("example.com")));
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_room_id() {
        let room_id = RoomId::new(server_name!("example.com"));
        let id_str = room_id.as_str();

        assert!(id_str.starts_with('!'));
        assert_eq!(id_str.len(), 31);
    }

    #[test]
    fn serialize_valid_room_id() {
        assert_eq!(
            serde_json::to_string(
                <&RoomId>::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id() {
        assert_eq!(
            serde_json::from_str::<OwnedRoomId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            <&RoomId>::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
        );
    }

    #[test]
    fn valid_room_id_with_explicit_standard_port() {
        let room_id =
            <&RoomId>::try_from("!29fhd83h92h0:example.com:443").expect("Failed to create RoomId.");
        assert_eq!(room_id, "!29fhd83h92h0:example.com:443");
        assert_eq!(room_id.server_name(), Some(server_name!("example.com:443")));
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            <&RoomId>::try_from("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId."),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(
            <&RoomId>::try_from("carl:example.com").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_server_name() {
        let room_id = <&RoomId>::try_from("!29fhd83h92h0").expect("Failed to create RoomId.");
        assert_eq!(room_id, "!29fhd83h92h0");
        assert_eq!(room_id.server_name(), None);
    }

    #[test]
    fn invalid_room_id_host() {
        let room_id = <&RoomId>::try_from("!29fhd83h92h0:/").expect("Failed to create RoomId.");
        assert_eq!(room_id, "!29fhd83h92h0:/");
        assert_eq!(room_id.server_name(), None);
    }

    #[test]
    fn invalid_room_id_port() {
        let room_id = <&RoomId>::try_from("!29fhd83h92h0:example.com:notaport")
            .expect("Failed to create RoomId.");
        assert_eq!(room_id, "!29fhd83h92h0:example.com:notaport");
        assert_eq!(room_id.server_name(), None);
    }
}
