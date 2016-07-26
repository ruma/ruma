//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, and users.

#![feature(question_mark)]
#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate url;

#[cfg(test)]
extern crate serde_json;

use std::fmt::{Display, Formatter, Result as FmtResult};

use regex::Regex;
use serde::{Serialize, Serializer};
use url::{ParseError, Url};

pub use url::Host;

/// All events must be 255 bytes or less.
const MAX_BYTES: usize = 255;
/// The minimum number of characters an ID can be.
///
/// This is an optimization and not required by the spec. The shortest possible valid ID is a sigil
/// + a single character local ID + a colon + a single character hostname.
const MIN_CHARS: usize = 4;
/// The number of bytes in a valid sigil.
const SIGIL_BYTES: usize = 1;

lazy_static! {
    static ref USER_LOCALPART_PATTERN: Regex =
        Regex::new(r"\A[a-z0-9._=-]+\z").expect("Failed to create user localpart regex.");
}

/// An error encountered when trying to parse an invalid ID string.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The ID's localpart contains invalid characters.
    ///
    /// Only relevant for user IDs.
    InvalidCharacters,
    /// The domain part of the the ID string is not a valid IP address or DNS name.
    InvalidHost,
    /// The ID exceeds 255 bytes.
    MaximumLengthExceeded,
    /// The ID is less than 4 characters.
    MinimumLengthNotSatisfied,
    /// The ID is missing the colon delimiter between localpart and server name.
    MissingDelimiter,
    /// The ID is missing the leading sigil.
    MissingSigil,
}

/// A Matrix event ID.
///
/// An `EventId` is created from a string slice, and can be converted back into a string as needed:
///
/// ```
/// # use ruma_identifiers::EventId;
/// assert_eq!(EventId::new("$h29iv0s8:example.com").unwrap().to_string(), "$h29iv0s8:example.com");
/// ```
#[derive(Debug)]
pub struct EventId {
    hostname: Host,
    opaque_id: String,
    port: u16,
}

/// A Matrix room alias ID.
///
/// A `RoomAliasId` is created from a string slice, and can be converted back into a string as
/// needed:
///
/// ```
/// # use ruma_identifiers::RoomAliasId;
/// assert_eq!(RoomAliasId::new("#ruma:example.com").unwrap().to_string(), "#ruma:example.com");
/// ```
#[derive(Debug)]
pub struct RoomAliasId {
    alias: String,
    hostname: Host,
    port: u16,
}

/// A Matrix room ID.
///
/// A `RoomId` is created from a string slice, and can be converted back into a string as needed:
///
/// ```
/// # use ruma_identifiers::RoomId;
/// assert_eq!(RoomId::new("!n8f893n9:example.com").unwrap().to_string(), "!n8f893n9:example.com");
/// ```
#[derive(Debug)]
pub struct RoomId {
    hostname: Host,
    opaque_id: String,
    port: u16,
}

/// A Matrix user ID.
///
/// A `UserId` is created from a string slice, and can be converted back into a string as needed:
///
/// ```
/// # use ruma_identifiers::UserId;
/// assert_eq!(UserId::new("@carl:example.com").unwrap().to_string(), "@carl:example.com");
/// ```
#[derive(Debug)]
pub struct UserId {
    hostname: Host,
    localpart: String,
    port: u16,
}

fn display(f: &mut Formatter, sigil: char, localpart: &str, hostname: &Host, port: u16)
-> FmtResult {
    if port == 443 {
        write!(f, "{}{}:{}", sigil, localpart, hostname)
    } else {
        write!(f, "{}{}:{}:{}", sigil, localpart, hostname, port)
    }
}

fn parse_id<'a>(required_sigil: char, id: &'a str) -> Result<(&'a str, Host, u16), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    let mut chars = id.chars();

    if id.len() < MIN_CHARS {
        return Err(Error::MinimumLengthNotSatisfied);
    }

    let sigil = chars.nth(0).expect("ID missing first character.");

    if sigil != required_sigil {
        return Err(Error::MissingSigil);
    }

    let delimiter_index = match chars.position(|c| c == ':') {
        Some(index) => index + 1,
        None => return Err(Error::MissingDelimiter),
    };

    let localpart = &id[1..delimiter_index];
    let raw_host = &id[delimiter_index + SIGIL_BYTES..];
    let url_string = format!("https://{}", raw_host);
    let url = Url::parse(&url_string)?;

    let host = match url.host() {
        Some(host) => host.to_owned(),
        None => return Err(Error::InvalidHost),
    };

    let port = url.port().unwrap_or(443);

    Ok((localpart, host, port))
}

impl EventId {
    /// Creates a new Matrix event ID from a string representation.
    ///
    /// The string must include the leading $ sigil, the opaque ID, a literal colon, and a valid
    /// server name.
    pub fn new(event_id: &str) -> Result<Self, Error> {
        let (opaque_id, host, port) = parse_id('$', event_id)?;

        Ok(EventId {
            hostname: host,
            opaque_id: opaque_id.to_owned(),
            port: port,
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

impl RoomId {
    /// Creates a new Matrix room ID from a string representation.
    ///
    /// The string must include the leading ! sigil, the opaque ID, a literal colon, and a valid
    /// server name.
    pub fn new(room_id: &str) -> Result<Self, Error> {
        let (opaque_id, host, port) = parse_id('!', room_id)?;

        Ok(RoomId {
            hostname: host,
            opaque_id: opaque_id.to_owned(),
            port: port,
        })
    }

    /// Returns a `Host` for the room ID, containing the server name (minus the port) of the
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

impl RoomAliasId {
    /// Creates a new Matrix room alias ID from a string representation.
    ///
    /// The string must include the leading # sigil, the alias, a literal colon, and a valid
    /// server name.
    pub fn new(room_id: &str) -> Result<Self, Error> {
        let (alias, host, port) = parse_id('#', room_id)?;

        Ok(RoomAliasId {
            alias: alias.to_owned(),
            hostname: host,
            port: port,
        })
    }

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

impl UserId {
    /// Creates a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a valid
    /// server name.
    pub fn new(user_id: &str) -> Result<UserId, Error> {
        let (localpart, host, port) = parse_id('@', user_id)?;

        if !USER_LOCALPART_PATTERN.is_match(localpart) {
            return Err(Error::InvalidCharacters);
        }

        Ok(UserId {
            hostname: host,
            port: port,
            localpart: localpart.to_owned(),
        })
    }

    /// Returns a `Host` for the user ID, containing the server name (minus the port) of the
    /// originating homeserver.
    ///
    /// The host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> &Host {
        &self.hostname
    }

    /// Returns the user's localpart.
    pub fn localpart(&self) -> &str {
        &self.localpart
    }

    /// Returns the port the originating homeserver can be accessed on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Error {
        Error::InvalidHost
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        display(f, '$', &self.opaque_id, &self.hostname, self.port)
    }
}

impl Display for RoomAliasId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        display(f, '#', &self.alias, &self.hostname, self.port)
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        display(f, '!', &self.opaque_id, &self.hostname, self.port)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        display(f, '@', &self.localpart, &self.hostname, self.port)
    }
}

impl Serialize for EventId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RoomAliasId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for UserId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;
    use super::{Error, EventId, RoomAliasId, RoomId, UserId};

    #[test]
    fn valid_event_id() {
        assert_eq!(
            EventId::new("$39hvsi03hlne:example.com")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn serialize_valid_event_id() {
        assert_eq!(
            to_string(
                &EventId::new("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
            ).expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn valid_event_id_with_explicit_standard_port() {
        assert_eq!(
            EventId::new("$39hvsi03hlne:example.com:443")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_event_id_with_non_standard_port() {
        assert_eq!(
            EventId::new("$39hvsi03hlne:example.com:5000")
                .expect("Failed to create EventId.")
                .to_string(),
            "$39hvsi03hlne:example.com:5000"
        );
    }

    #[test]
    fn missing_event_id_sigil() {
        assert_eq!(
            EventId::new("39hvsi03hlne:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_event_id_delimiter() {
        assert_eq!(
            EventId::new("$39hvsi03hlne").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_event_id_host() {
        assert_eq!(
            EventId::new("$39hvsi03hlne:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_event_id_port() {
        assert_eq!(
            EventId::new("$39hvsi03hlne:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn valid_room_alias_id() {
        assert_eq!(
            RoomAliasId::new("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn serialize_valid_room_alias_id() {
        assert_eq!(
            to_string(
                &RoomAliasId::new("#ruma:example.com").expect("Failed to create RoomAliasId.")
            ).expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn valid_room_alias_id_with_explicit_standard_port() {
        assert_eq!(
            RoomAliasId::new("#ruma:example.com:443")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_alias_id_with_non_standard_port() {
        assert_eq!(
            RoomAliasId::new("#ruma:example.com:5000")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com:5000"
        );
    }

    #[test]
    fn missing_room_alias_id_sigil() {
        assert_eq!(
            RoomAliasId::new("39hvsi03hlne:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_room_alias_id_delimiter() {
        assert_eq!(
            RoomAliasId::new("#ruma").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_room_alias_id_host() {
        assert_eq!(
            RoomAliasId::new("#ruma:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_alias_id_port() {
        assert_eq!(
            RoomAliasId::new("#ruma:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }
    #[test]
    fn valid_room_id() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn serialize_valid_room_id() {
        assert_eq!(
            to_string(
                &RoomId::new("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
            ).expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn valid_room_id_with_explicit_standard_port() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0:example.com:443")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(
            RoomId::new("carl:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_room_id_delimiter() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_room_id_host() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_id_port() {
        assert_eq!(
            RoomId::new("!29fhd83h92h0:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn valid_user_id() {
        assert_eq!(
            UserId::new("@carl:example.com")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com"
        );
    }

    #[test]
    fn serialize_valid_user_id() {
        assert_eq!(
            to_string(
                &UserId::new("@carl:example.com").expect("Failed to create UserId.")
            ).expect("Failed to convert UserId to JSON."),
            r#""@carl:example.com""#
        );
    }

    #[test]
    fn valid_user_id_with_explicit_standard_port() {
        assert_eq!(
            UserId::new("@carl:example.com:443")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com"
        );
    }

    #[test]
    fn valid_user_id_with_non_standard_port() {
        assert_eq!(
            UserId::new("@carl:example.com:5000")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com:5000"
        );
    }

    #[test]
    fn invalid_characters_in_user_id_localpart() {
        assert_eq!(
            UserId::new("@CARL:example.com").err().unwrap(),
            Error::InvalidCharacters
        );
    }

    #[test]
    fn missing_user_id_sigil() {
        assert_eq!(
            UserId::new("carl:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_user_id_delimiter() {
        assert_eq!(
            UserId::new("@carl").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_user_id_host() {
        assert_eq!(
            UserId::new("@carl:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            UserId::new("@carl:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }
}
