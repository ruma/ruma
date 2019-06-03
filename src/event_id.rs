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
/// # Room versions
///
/// Matrix specifies multiple [room versions](https://matrix.org/docs/spec/#room-versions) and the
/// format of event identifiers differ between them. The original format used by room versions 1
/// and 2 uses a short pseudorandom "localpart" followed by the hostname and port of the
/// originating homeserver. Later room versions change event identifiers to be a hash of the event
/// encoded with Base64. Some of the methods provided by `EventId` are only relevant to the
/// original event format.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::EventId;
/// // Original format
/// assert_eq!(
///     EventId::try_from("$h29iv0s8:example.com").unwrap().to_string(),
///     "$h29iv0s8:example.com"
/// );
/// // Room version 3 format
/// assert_eq!(
///     EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap().to_string(),
///     "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
/// );
/// // Room version 4 format
/// assert_eq!(
///     EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap().to_string(),
///     "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct EventId(Format);

/// Different event ID formats from the different Matrix room versions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Format {
    /// The original format as used by Matrix room versions 1 and 2.
    Original(Original),
    /// The format used by Matrix room version 3.
    Base64(String),
    /// The format used by Matrix room version 4.
    UrlSafeBase64(String),
}

/// An event in the original format as used by Matrix room versions 1 and 2.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Original {
    /// The hostname of the homeserver.
    pub hostname: Host,
    /// The event's unique ID.
    pub localpart: String,
    /// The network port of the homeserver.
    pub port: u16,
}

/// A serde visitor for `EventId`.
struct EventIdVisitor;

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters. This should only be used for events in the original format
    /// as used by Matrix room versions 1 and 2.
    ///
    /// Fails if the homeserver cannot be parsed as a valid host.
    pub fn new(homeserver_host: &str) -> Result<Self, Error> {
        let event_id = format!("${}:{}", generate_localpart(18), homeserver_host);
        let (localpart, host, port) = parse_id('$', &event_id)?;

        Ok(Self(Format::Original(Original {
            hostname: host,
            localpart: localpart.to_string(),
            port,
        })))
    }

    /// Returns a `Host` for the event ID, containing the server name (minus the port) of the
    /// originating homeserver. Only applicable to events in the original format as used by Matrix
    /// room versions 1 and 2.
    ///
    /// The host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> Option<&Host> {
        if let Format::Original(original) = &self.0 {
            Some(&original.hostname)
        } else {
            None
        }
    }

    /// Returns the event's unique ID. For the original event format as used by Matrix room
    /// versions 1 and 2, this is the "localpart" that precedes the homeserver. For later formats,
    /// this is the entire ID without the leading $ sigil.
    pub fn localpart(&self) -> &str {
        match &self.0 {
            Format::Original(original) => &original.localpart,
            Format::Base64(id) | Format::UrlSafeBase64(id) => id,
        }
    }

    /// Returns the port the originating homeserver can be accessed on. Only applicable to events
    /// in the original format as used by Matrix room versions 1 and 2.
    pub fn port(&self) -> Option<u16> {
        if let Format::Original(original) = &self.0 {
            Some(original.port)
        } else {
            None
        }
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            Format::Original(original) => display(
                f,
                '$',
                &original.localpart,
                &original.hostname,
                original.port,
            ),
            Format::Base64(id) | Format::UrlSafeBase64(id) => write!(f, "${}", id),
        }
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
    /// If using the original event format as used by Matrix room versions 1 and 2, the string must
    /// include the leading $ sigil, the localpart, a literal colon, and a valid homeserver
    /// hostname.
    fn try_from(event_id: &'a str) -> Result<Self, Self::Error> {
        if event_id.contains(':') {
            let (localpart, host, port) = parse_id('$', event_id)?;

            Ok(Self(Format::Original(Original {
                hostname: host,
                localpart: localpart.to_owned(),
                port,
            })))
        } else if !event_id.starts_with('$') {
            Err(Error::MissingSigil)
        } else if event_id.contains(|chr| chr == '+' || chr == '/') {
            Ok(Self(Format::Base64(event_id[1..].to_string())))
        } else {
            Ok(Self(Format::UrlSafeBase64(event_id[1..].to_string())))
        }
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
    fn valid_original_event_id() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_base64_event_id() {
        assert_eq!(
            EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
                .to_string(),
            "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
        )
    }

    #[test]
    fn valid_url_safe_base64_event_id() {
        assert_eq!(
            EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId.")
                .to_string(),
            "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
        )
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
    fn serialize_valid_original_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn serialize_valid_base64_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#
        );
    }

    #[test]
    fn serialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#
        );
    }

    #[test]
    fn deserialize_valid_original_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$39hvsi03hlne:example.com""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_base64_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn valid_original_event_id_with_explicit_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:443")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_original_event_id_with_non_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:5000")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com:5000"
        );
    }

    #[test]
    fn missing_original_event_id_sigil() {
        assert_eq!(
            EventId::try_from("39hvsi03hlne:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .err()
                .unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_url_safe_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .err()
                .unwrap(),
            Error::MissingSigil
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
