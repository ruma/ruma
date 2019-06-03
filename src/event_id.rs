//! Matrix event identifiers.

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

/// A Matrix event ID.
///
/// An `EventId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::EventId;
/// assert_eq!(
///     EventId::try_from("$h29iv0s8:example.com").unwrap().to_string(),
///     "$h29iv0s8:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct EventId {
    /// The hostname of the homeserver.
    hostname: Host,
    /// The event's unique ID.
    opaque_id: String,
    /// The network port of the homeserver.
    port: u16,
}

/// A serde visitor for `EventId`.
struct EventIdVisitor;

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters.
    ///
    /// Fails if the given origin server name cannot be parsed as a valid host.
    pub fn new(server_name: &str) -> Result<Self, Error> {
        let event_id = format!("${}:{}", generate_localpart(18), server_name);
        let (opaque_id, host, port) = parse_id('$', &event_id)?;

        Ok(Self {
            hostname: host,
            opaque_id: opaque_id.to_string(),
            port,
        })
    }

    /// Returns a `Host` for the event ID, containing the server name (minus the port) of the
    /// originating homeserver.
    ///
    /// The host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> &Host {
        &self.hostname
    }

    /// Returns the event's opaque ID.
    pub fn opaque_id(&self) -> &str {
        &self.opaque_id
    }

    /// Returns the port the originating homeserver can be accessed on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display(f, '$', &self.opaque_id, &self.hostname, self.port)
    }
}

impl Serialize for EventId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for EventId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(EventIdVisitor)
    }
}

impl<'a> TryFrom<&'a str> for EventId {
    type Error = Error;

    /// Attempts to create a new Matrix event ID from a string representation.
    ///
    /// The string must include the leading $ sigil, the opaque ID, a literal colon, and a valid
    /// server name.
    fn try_from(event_id: &'a str) -> Result<Self, Self::Error> {
        let (opaque_id, host, port) = parse_id('$', event_id)?;

        Ok(Self {
            hostname: host,
            opaque_id: opaque_id.to_owned(),
            port,
        })
    }
}

impl<'de> Visitor<'de> for EventIdVisitor {
    type Value = EventId;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a Matrix event ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        match EventId::try_from(v) {
            Ok(event_id) => Ok(event_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::EventId;
    use crate::error::Error;

    #[test]
    fn valid_event_id() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn generate_random_valid_event_id() {
        let event_id = EventId::new("example.com")
            .expect("Failed to generate EventId.")
            .to_string();

        assert!(event_id.to_string().starts_with('$'));
        assert_eq!(event_id.len(), 31);
    }

    #[test]
    fn generate_random_invalid_event_id() {
        assert!(EventId::new("").is_err());
    }

    #[test]
    fn serialize_valid_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$39hvsi03hlne:example.com""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
        );
    }

    #[test]
    fn valid_event_id_with_explicit_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:443")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_event_id_with_non_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:5000")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com:5000"
        );
    }

    #[test]
    fn missing_event_id_sigil() {
        assert_eq!(
            EventId::try_from("39hvsi03hlne:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_event_id_delimiter() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_event_id_host() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_event_id_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:notaport")
                .err()
                .unwrap(),
            Error::InvalidHost
        );
    }
}
