//! Matrix room alias identifiers.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;
use serde::{
    de::{Error as SerdeError, Expected, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use url::Host;

use crate::{display, error::Error, parse_id};

/// A Matrix room alias ID.
///
/// A `RoomAliasId` is converted from a string slice, and can be converted back into a string as
/// needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomAliasId;
/// assert_eq!(
///     RoomAliasId::try_from("#ruma:example.com").unwrap().to_string(),
///     "#ruma:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct RoomAliasId {
    /// The alias for the room.
    alias: String,
    /// The hostname of the homeserver.
    hostname: Host,
    /// The network port of the homeserver.
    port: u16,
}

impl RoomAliasId {
    /// Returns a `Host` for the room alias ID, containing the server name (minus the port) of
    /// the originating homeserver.
    ///
    /// The host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> &Host {
        &self.hostname
    }

    /// Returns the room's alias.
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Returns the port the originating homeserver can be accessed on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Display for RoomAliasId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display(f, '#', &self.alias, &self.hostname, self.port)
    }
}

impl Serialize for RoomAliasId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RoomAliasId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|v| {
            RoomAliasId::try_from(&v as &str)
                .map_err(|_| SerdeError::invalid_value(Unexpected::Str(&v), &ExpectedRoomAliasId))
        })
    }
}

impl<'a> TryFrom<&'a str> for RoomAliasId {
    type Error = Error;

    /// Attempts to create a new Matrix room alias ID from a string representation.
    ///
    /// The string must include the leading # sigil, the alias, a literal colon, and a valid
    /// server name.
    fn try_from(room_id: &'a str) -> Result<Self, Error> {
        let (alias, host, port) = parse_id('#', room_id)?;

        Ok(Self {
            alias: alias.to_owned(),
            hostname: host,
            port,
        })
    }
}

struct ExpectedRoomAliasId;

impl Expected for ExpectedRoomAliasId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a Matrix room alias ID as a string")
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::RoomAliasId;
    use crate::error::Error;

    #[test]
    fn valid_room_alias_id() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn serialize_valid_room_alias_id() {
        assert_eq!(
            to_string(
                &RoomAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
            )
            .expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn deserialize_valid_room_alias_id() {
        assert_eq!(
            from_str::<RoomAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            RoomAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn valid_room_alias_id_with_explicit_standard_port() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:example.com:443")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_alias_id_with_non_standard_port() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:example.com:5000")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com:5000"
        );
    }

    #[test]
    fn valid_room_alias_id_unicode() {
        assert_eq!(
            RoomAliasId::try_from("#老虎Â£я:example.com")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#老虎Â£я:example.com"
        );
    }

    #[test]
    fn missing_room_alias_id_sigil() {
        assert_eq!(
            RoomAliasId::try_from("39hvsi03hlne:example.com")
                .err()
                .unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_room_alias_id_delimiter() {
        assert_eq!(
            RoomAliasId::try_from("#ruma").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_room_alias_id_host() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:/").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_alias_id_port() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:example.com:notaport")
                .err()
                .unwrap(),
            Error::InvalidHost
        );
    }
}
