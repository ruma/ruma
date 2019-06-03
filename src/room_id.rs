//! Matrix room identifiers.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;
use serde::{
    de::{Error as SerdeError, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use url::Host;

use crate::{display, error::Error, generate_localpart, parse_id};

/// A Matrix room ID.
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomId;
/// assert_eq!(
///     RoomId::try_from("!n8f893n9:example.com").unwrap().to_string(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct RoomId {
    /// The hostname of the homeserver.
    hostname: Host,
    /// The room's unique ID.
    localpart: String,
    /// The network port of the homeserver.
    port: u16,
}

/// A serde visitor for `RoomId`.
struct RoomIdVisitor;

impl RoomId {
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    pub fn new(homeserver_host: &str) -> Result<Self, Error> {
        let room_id = format!("!{}:{}", generate_localpart(18), homeserver_host);
        let (localpart, host, port) = parse_id('!', &room_id)?;

        Ok(Self {
            hostname: host,
            localpart: localpart.to_string(),
            port,
        })
    }

    /// Returns a `Host` for the room ID, containing the server name (minus the port) of the
    /// originating homeserver.
    ///
    /// The host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> &Host {
        &self.hostname
    }

    /// Returns the rooms's unique ID.
    pub fn localpart(&self) -> &str {
        &self.localpart
    }

    /// Returns the port the originating homeserver can be accessed on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display(f, '!', &self.localpart, &self.hostname, self.port)
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RoomIdVisitor)
    }
}

impl<'a> TryFrom<&'a str> for RoomId {
    type Error = Error;

    /// Attempts to create a new Matrix room ID from a string representation.
    ///
    /// The string must include the leading ! sigil, the localpart, a literal colon, and a valid
    /// server name.
    fn try_from(room_id: &'a str) -> Result<Self, Error> {
        let (localpart, host, port) = parse_id('!', room_id)?;

        Ok(Self {
            hostname: host,
            localpart: localpart.to_owned(),
            port,
        })
    }
}

impl<'de> Visitor<'de> for RoomIdVisitor {
    type Value = RoomId;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a Matrix room ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        match RoomId::try_from(v) {
            Ok(room_id) => Ok(room_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

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
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn generate_random_valid_room_id() {
        let room_id = RoomId::new("example.com")
            .expect("Failed to generate RoomId.")
            .to_string();

        assert!(room_id.to_string().starts_with('!'));
        assert_eq!(room_id.len(), 31);
    }

    #[test]
    fn generate_random_invalid_room_id() {
        assert!(RoomId::new("").is_err());
    }

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
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(
            RoomId::try_from("carl:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_room_id_delimiter() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_room_id_host() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_id_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:notaport")
                .err()
                .unwrap(),
            Error::InvalidHost
        );
    }
}
