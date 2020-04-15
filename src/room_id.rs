//! Matrix room identifiers.

use std::{borrow::Cow, convert::TryFrom, num::NonZeroU8};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;

use crate::{error::Error, generate_localpart, parse_id};

/// A Matrix room ID.
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomId;
/// assert_eq!(
///     RoomId::try_from("!n8f893n9:example.com").unwrap().as_ref(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct RoomId {
    full_id: String,
    colon_idx: NonZeroU8,
}

impl RoomId {
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    pub fn new(homeserver_host: &str) -> Result<Self, Error> {
        let full_id = format!("!{}:{}", generate_localpart(18), homeserver_host);

        Ok(Self {
            full_id,
            colon_idx: NonZeroU8::new(19).unwrap(),
        })
    }

    /// Returns the host of the room ID, containing the server name (including the port) of the
    /// originating homeserver.
    pub fn hostname(&self) -> &str {
        &self.full_id[self.colon_idx.get() as usize + 1..]
    }

    /// Returns the rooms's unique ID.
    pub fn localpart(&self) -> &str {
        &self.full_id[1..self.colon_idx.get() as usize]
    }
}

impl TryFrom<Cow<'_, str>> for RoomId {
    type Error = Error;

    /// Attempts to create a new Matrix room ID from a string representation.
    ///
    /// The string must include the leading ! sigil, the localpart, a literal colon, and a server
    /// name.
    fn try_from(room_id: Cow<'_, str>) -> Result<Self, Error> {
        let colon_idx = parse_id(&room_id, &['!'])?;

        Ok(Self {
            full_id: room_id.into_owned(),
            colon_idx,
        })
    }
}

common_impls!(RoomId, "a Matrix room ID");

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::RoomId;
    use crate::error::Error;

    #[test]
    fn valid_room_id() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn generate_random_valid_room_id() {
        let room_id = RoomId::new("example.com").expect("Failed to generate RoomId.");
        let id_str: &str = room_id.as_ref();

        assert!(id_str.starts_with('!'));
        assert_eq!(id_str.len(), 31);
    }

    /*#[test]
    fn generate_random_invalid_room_id() {
        assert!(RoomId::new("").is_err());
    }*/

    #[test]
    fn serialize_valid_room_id() {
        assert_eq!(
            to_string(
                &RoomId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id() {
        assert_eq!(
            from_str::<RoomId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            RoomId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
        );
    }

    #[test]
    fn valid_room_id_with_explicit_standard_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:443")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:443"
        );
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(
            RoomId::try_from("carl:example.com").unwrap_err(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_room_id_delimiter() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0").unwrap_err(),
            Error::MissingDelimiter
        );
    }

    /*#[test]
    fn invalid_room_id_host() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:/").unwrap_err(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_id_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:notaport").unwrap_err(),
            Error::InvalidHost
        );
    }*/
}
