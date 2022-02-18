//! Matrix room identifiers.

use crate::{EventId, MatrixToUri, ServerName};

/// A Matrix room ID.
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomId;
/// assert_eq!(<&RoomId>::try_from("!n8f893n9:example.com").unwrap(), "!n8f893n9:example.com");
/// ```
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(str);

opaque_identifier_validated!(RoomId, ruma_identifiers_validation::room_id::validate);

impl RoomId {
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    #[cfg(feature = "rand")]
    pub fn new(server_name: &ServerName) -> Box<Self> {
        Self::from_owned(format!("!{}:{}", crate::generate_localpart(18), server_name).into())
    }

    /// Returns the rooms's unique ID.
    pub fn localpart(&self) -> &str {
        &self.as_str()[1..self.colon_idx()]
    }

    /// Returns the server name of the room ID.
    pub fn server_name(&self) -> &ServerName {
        ServerName::from_borrowed(&self.as_str()[self.colon_idx() + 1..])
    }

    /// Create a `matrix.to` reference for this room ID.
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_identifiers::{room_id, server_name};
    ///
    /// assert_eq!(
    ///     room_id!("!somewhere:example.org")
    ///         .matrix_to_url([&*server_name!("example.org"), &*server_name!("alt.example.org")])
    ///         .to_string(),
    ///     "https://matrix.to/#/%21somewhere%3Aexample.org?via=example.org&via=alt.example.org"
    /// );
    /// ```
    pub fn matrix_to_url<'a>(
        &'a self,
        via: impl IntoIterator<Item = &'a ServerName>,
    ) -> MatrixToUri<'a> {
        MatrixToUri::new(self.as_str(), via.into_iter().collect())
    }

    /// Create a `matrix.to` reference for an event scoped under this room ID.
    pub fn matrix_to_event_url<'a>(&'a self, ev_id: &'a EventId) -> MatrixToUri<'a> {
        MatrixToUri::event(self.as_str(), ev_id, Vec::new())
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::RoomId;
    use crate::Error;

    #[test]
    fn valid_room_id() {
        assert_eq!(
            <&RoomId>::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn empty_localpart() {
        assert_eq!(
            <&RoomId>::try_from("!:example.com").expect("Failed to create RoomId.").as_ref(),
            "!:example.com"
        );
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_room_id() {
        use crate::server_name;

        let room_id = RoomId::new(server_name!("example.com"));
        let id_str = room_id.as_str();

        assert!(id_str.starts_with('!'));
        assert_eq!(id_str.len(), 31);
    }

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_room_id() {
        assert_eq!(
            serde_json::from_str::<Box<RoomId>>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            <&RoomId>::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
        );
    }

    #[test]
    fn valid_room_id_with_explicit_standard_port() {
        assert_eq!(
            <&RoomId>::try_from("!29fhd83h92h0:example.com:443")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:443"
        );
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            <&RoomId>::try_from("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(
            <&RoomId>::try_from("carl:example.com").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_room_id_delimiter() {
        assert_eq!(<&RoomId>::try_from("!29fhd83h92h0").unwrap_err(), Error::MissingDelimiter);
    }

    #[test]
    fn invalid_room_id_host() {
        assert_eq!(<&RoomId>::try_from("!29fhd83h92h0:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_room_id_port() {
        assert_eq!(
            <&RoomId>::try_from("!29fhd83h92h0:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
