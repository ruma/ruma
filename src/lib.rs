//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, and users.

#![feature(try_from)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate url;

#[cfg(feature = "diesel")]
extern crate diesel;

#[cfg(test)]
extern crate serde_json;

use std::error::Error as StdError;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

use rand::{Rng, thread_rng};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error as SerdeError, Unexpected, Visitor};
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
#[derive(Copy, Clone, Debug, PartialEq)]
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
/// An `EventId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # #![feature(try_from)]
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::EventId;
/// assert_eq!(
///     EventId::try_from("$h29iv0s8:example.com").unwrap().to_string(),
///     "$h29iv0s8:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EventId {
    hostname: Host,
    opaque_id: String,
    port: u16,
}

/// A Matrix room alias ID.
///
/// A `RoomAliasId` is converted from a string slice, and can be converted back into a string as
/// needed.
///
/// ```
/// # #![feature(try_from)]
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomAliasId;
/// assert_eq!(
///     RoomAliasId::try_from("#ruma:example.com").unwrap().to_string(),
///     "#ruma:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RoomAliasId {
    alias: String,
    hostname: Host,
    port: u16,
}

/// A Matrix room ID.
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # #![feature(try_from)]
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomId;
/// assert_eq!(
///     RoomId::try_from("!n8f893n9:example.com").unwrap().to_string(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RoomId {
    hostname: Host,
    opaque_id: String,
    port: u16,
}

/// A Matrix room ID or a Matrix room alias ID.
///
/// `RoomIdOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # #![feature(try_from)]
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomIdOrAliasId;
/// assert_eq!(
///     RoomIdOrAliasId::try_from("#ruma:example.com").unwrap().to_string(),
///     "#ruma:example.com"
/// );
///
/// assert_eq!(
///     RoomIdOrAliasId::try_from("!n8f893n9:example.com").unwrap().to_string(),
///     "!n8f893n9:example.com"
/// );
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RoomIdOrAliasId {
    /// A Matrix room alias ID.
    RoomAliasId(RoomAliasId),
    /// A Matrix room ID.
    RoomId(RoomId),
}

/// A Matrix user ID.
///
/// A `UserId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # #![feature(try_from)]
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::UserId;
/// assert_eq!(
///     UserId::try_from("@carl:example.com").unwrap().to_string(),
///     "@carl:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserId {
    hostname: Host,
    localpart: String,
    port: u16,
}

struct EventIdVisitor;
struct RoomAliasIdVisitor;
struct RoomIdVisitor;
struct RoomIdOrAliasIdVisitor;
struct UserIdVisitor;

fn display(f: &mut Formatter, sigil: char, localpart: &str, hostname: &Host, port: u16)
-> FmtResult {
    if port == 443 {
        write!(f, "{}{}:{}", sigil, localpart, hostname)
    } else {
        write!(f, "{}{}:{}:{}", sigil, localpart, hostname, port)
    }
}

fn generate_localpart(length: usize) -> String {
    thread_rng().gen_ascii_chars().take(length).collect()
}

fn validate_id<'a>(id: &'a str) -> Result<(), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    if id.len() < MIN_CHARS {
        return Err(Error::MinimumLengthNotSatisfied);
    }

    Ok(())
}

fn parse_id<'a>(required_sigil: char, id: &'a str) -> Result<(&'a str, Host, u16), Error> {
    validate_id(id)?;

    let mut chars = id.chars();

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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidCharacters => "localpart contains invalid characters",
            Error::InvalidHost => "server name is not a valid IP address or domain name",
            Error::MaximumLengthExceeded => "ID exceeds 255 bytes",
            Error::MinimumLengthNotSatisfied => "ID must be at least 4 characters",
            Error::MissingDelimiter => "colon is required between localpart and server name",
            Error::MissingSigil => "leading sigil is missing",
        }
    }
}

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters.
    ///
    /// Fails if the given origin server name cannot be parsed as a valid host.
    pub fn new(server_name: &str) -> Result<Self, Error> {
        let event_id = format!("${}:{}", generate_localpart(18), server_name);
        let (opaque_id, host, port) = parse_id('$', &event_id)?;

        Ok(EventId {
            hostname: host,
            opaque_id: opaque_id.to_string(),
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
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given origin server name cannot be parsed as a valid host.
    pub fn new(server_name: &str) -> Result<Self, Error> {
        let room_id = format!("!{}:{}", generate_localpart(18), server_name);
        let (opaque_id, host, port) = parse_id('!', &room_id)?;

        Ok(RoomId {
            hostname: host,
            opaque_id: opaque_id.to_string(),
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
    /// Attempts to generate a `UserId` for the given origin server with a localpart consisting of
    /// 12 random ASCII characters.
    ///
    /// Fails if the given origin server name cannot be parsed as a valid host.
    pub fn new(server_name: &str) -> Result<Self, Error> {
        let user_id = format!("@{}:{}", generate_localpart(12).to_lowercase(), server_name);
        let (localpart, host, port) = parse_id('@', &user_id)?;

        Ok(UserId {
            hostname: host,
            localpart: localpart.to_string(),
            port: port,
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

impl Display for RoomIdOrAliasId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            RoomIdOrAliasId::RoomAliasId(ref room_alias_id) => {
                display(f, '#', &room_alias_id.alias, &room_alias_id.hostname, room_alias_id.port)
            }
            RoomIdOrAliasId::RoomId(ref room_id) => {
                display(f, '!', &room_id.opaque_id, &room_id.hostname, room_id.port)
            }
        }
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        display(f, '@', &self.localpart, &self.hostname, self.port)
    }
}

impl Serialize for EventId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RoomAliasId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RoomIdOrAliasId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match *self {
            RoomIdOrAliasId::RoomAliasId(ref room_alias_id) => {
                serializer.serialize_str(&room_alias_id.to_string())
            }
            RoomIdOrAliasId::RoomId(ref room_id) => {
                serializer.serialize_str(&room_id.to_string())
            }
        }
    }
}

impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for EventId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(EventIdVisitor)
    }
}

impl Deserialize for RoomAliasId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(RoomAliasIdVisitor)
    }
}

impl Deserialize for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(RoomIdVisitor)
    }
}

impl Deserialize for RoomIdOrAliasId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(RoomIdOrAliasIdVisitor)
    }
}

impl Deserialize for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(UserIdVisitor)
    }
}

impl<'a> TryFrom<&'a str> for EventId {
    type Err = Error;

    /// Attempts to create a new Matrix event ID from a string representation.
    ///
    /// The string must include the leading $ sigil, the opaque ID, a literal colon, and a valid
    /// server name.
    fn try_from(event_id: &'a str) -> Result<Self, Self::Err> {
        let (opaque_id, host, port) = parse_id('$', event_id)?;

        Ok(EventId {
            hostname: host,
            opaque_id: opaque_id.to_owned(),
            port: port,
        })
    }
}

impl<'a> TryFrom<&'a str> for RoomAliasId {
    type Err = Error;

    /// Attempts to create a new Matrix room alias ID from a string representation.
    ///
    /// The string must include the leading # sigil, the alias, a literal colon, and a valid
    /// server name.
    fn try_from(room_id: &'a str) -> Result<Self, Error> {
        let (alias, host, port) = parse_id('#', room_id)?;

        Ok(RoomAliasId {
            alias: alias.to_owned(),
            hostname: host,
            port: port,
        })
    }
}

impl<'a> TryFrom<&'a str> for RoomId {
    type Err = Error;

    /// Attempts to create a new Matrix room ID from a string representation.
    ///
    /// The string must include the leading ! sigil, the opaque ID, a literal colon, and a valid
    /// server name.
    fn try_from(room_id: &'a str) -> Result<Self, Error> {
        let (opaque_id, host, port) = parse_id('!', room_id)?;

        Ok(RoomId {
            hostname: host,
            opaque_id: opaque_id.to_owned(),
            port: port,
        })
    }
}

impl<'a> TryFrom<&'a str> for RoomIdOrAliasId {
    type Err = Error;

    /// Attempts to create a new Matrix room ID or a room alias ID from a string representation.
    ///
    /// The string must either
    /// include the leading ! sigil, the opaque ID, a literal colon, and a valid server name or
    /// include the leading # sigil, the alias, a literal colon, and a valid server name.
    fn try_from(room_id_or_alias_id: &'a str) -> Result<Self, Error> {
        validate_id(room_id_or_alias_id)?;

        let mut chars = room_id_or_alias_id.chars();

        let sigil = chars.nth(0).expect("ID missing first character.");

        match sigil {
            '#' => {
                let room_alias_id = RoomAliasId::try_from(room_id_or_alias_id)?;
                Ok(RoomIdOrAliasId::RoomAliasId(room_alias_id))
            }
            '!' => {
                let room_id = RoomId::try_from(room_id_or_alias_id)?;
                Ok(RoomIdOrAliasId::RoomId(room_id))
            }
            _ => Err(Error::MissingSigil)
        }
    }
}

impl<'a> TryFrom<&'a str> for UserId {
    type Err = Error;

    /// Attempts to create a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a valid
    /// server name.
    fn try_from(user_id: &'a str) -> Result<UserId, Error> {
        let (localpart, host, port) = parse_id('@', user_id)?;
        let downcased_localpart = localpart.to_lowercase();

        if !USER_LOCALPART_PATTERN.is_match(&downcased_localpart) {
            return Err(Error::InvalidCharacters);
        }

        Ok(UserId {
            hostname: host,
            port: port,
            localpart: downcased_localpart.to_owned(),
        })
    }
}

impl Visitor for EventIdVisitor {
    type Value = EventId;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a Matrix event ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
        match EventId::try_from(v) {
            Ok(event_id) => Ok(event_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl Visitor for RoomAliasIdVisitor {
    type Value = RoomAliasId;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a Matrix room alias ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
        match RoomAliasId::try_from(v) {
            Ok(room_alias_id) => Ok(room_alias_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl Visitor for RoomIdVisitor {
    type Value = RoomId;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a Matrix room ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
        match RoomId::try_from(v) {
            Ok(room_id) => Ok(room_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl Visitor for RoomIdOrAliasIdVisitor {
    type Value = RoomIdOrAliasId;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a Matrix room ID or room alias ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
        match RoomIdOrAliasId::try_from(v) {
            Ok(room_id_or_alias_id) => Ok(room_id_or_alias_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl Visitor for UserIdVisitor {
    type Value = UserId;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a Matrix user ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
        match UserId::try_from(v) {
            Ok(user_id) => Ok(user_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_integration {
    use std::convert::TryFrom;
    use std::error::Error;
    use std::io::Write;

    use diesel::Queryable;
    use diesel::backend::Backend;
    use diesel::expression::AsExpression;
    use diesel::expression::bound::Bound;
    use diesel::row::Row;
    use diesel::types::{FromSql, FromSqlRow, HasSqlType, IsNull, Text, ToSql};

    macro_rules! diesel_impl {
        ($name:ident) => {
            impl<A, DB> FromSql<A, DB> for $crate::$name
            where DB: Backend + HasSqlType<A>, String: FromSql<A, DB> {
                fn from_sql(bytes: Option<&DB::RawValue>)
                -> Result<Self, Box<Error + Send + Sync>> {
                    let string = <String as FromSql<A, DB>>::from_sql(bytes)?;

                    $crate::$name::try_from(&string)
                        .map_err(|error| Box::new(error) as Box<Error + Send + Sync>)
                }
            }

            impl<A, DB> FromSqlRow<A, DB> for $crate::$name
            where DB: Backend + HasSqlType<A>, String: FromSql<A, DB> {
                fn build_from_row<T: Row<DB>>(row: &mut T)
                -> Result<Self, Box<Error + Send + Sync>> {
                    FromSql::<A, DB>::from_sql(row.take())
                }
            }

            impl<A, DB> ToSql<A, DB> for $crate::$name
            where DB: Backend + HasSqlType<A>, String: ToSql<A, DB> {
                fn to_sql<W: Write>(&self, out: &mut W)
                -> Result<IsNull, Box<Error + Send + Sync>> {
                    self.to_string().to_sql(out)
                }
            }

            impl<A, DB> Queryable<A, DB> for $crate::$name where
                $crate::$name: FromSqlRow<A, DB>,
                DB: Backend + HasSqlType<A>,
            {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }

            impl AsExpression<Text> for $crate::$name {
                type Expression = Bound<Text, Self>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl<'a> AsExpression<Text> for &'a $crate::$name {
                type Expression = Bound<Text, Self>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }
        }
    }

    diesel_impl!(EventId);
    diesel_impl!(RoomAliasId);
    diesel_impl!(RoomId);
    diesel_impl!(RoomIdOrAliasId);
    diesel_impl!(UserId);
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::{Error, EventId, RoomAliasId, RoomId, RoomIdOrAliasId, UserId};

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
            ).expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_event_id() {
        assert_eq!(
            from_str::<EventId>(
                r#""$39hvsi03hlne:example.com""#
            ).expect("Failed to convert JSON to EventId"),
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
            EventId::try_from("$39hvsi03hlne:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }

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
            ).expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn deserialize_valid_room_alias_id() {
        assert_eq!(
            from_str::<RoomAliasId>(
                r##""#ruma:example.com""##
            ).expect("Failed to convert JSON to RoomAliasId"),
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
    fn missing_room_alias_id_sigil() {
        assert_eq!(
            RoomAliasId::try_from("39hvsi03hlne:example.com").err().unwrap(),
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
            RoomAliasId::try_from("#ruma:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_room_alias_id_port() {
        assert_eq!(
            RoomAliasId::try_from("#ruma:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }
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
            ).expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id() {
        assert_eq!(
            from_str::<RoomId>(
                r#""!29fhd83h92h0:example.com""#
            ).expect("Failed to convert JSON to RoomId"),
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
            RoomId::try_from("!29fhd83h92h0:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn missing_sigil_for_room_id_or_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("ruma:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            to_string(
                &RoomIdOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
            ).expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            to_string(
                &RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
            ).expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            from_str::<RoomIdOrAliasId>(
                r##""#ruma:example.com""##
            ).expect("Failed to convert JSON to RoomAliasId"),
            RoomIdOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            from_str::<RoomIdOrAliasId>(
                r##""!29fhd83h92h0:example.com""##
            ).expect("Failed to convert JSON to RoomId"),
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn valid_user_id() {
        assert_eq!(
            UserId::try_from("@carl:example.com")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com"
        );
    }

    #[test]
    fn downcase_user_id() {
        assert_eq!(
            UserId::try_from("@CARL:example.com")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com"
        );
    }

    #[test]
    fn generate_random_valid_user_id() {
        let user_id = UserId::new("example.com")
            .expect("Failed to generate UserId.")
            .to_string();

        assert!(user_id.to_string().starts_with('@'));
        assert_eq!(user_id.len(), 25);
    }

    #[test]
    fn generate_random_invalid_user_id() {
        assert!(UserId::new("").is_err());
    }

    #[test]
    fn serialize_valid_user_id() {
        assert_eq!(
            to_string(
                &UserId::try_from("@carl:example.com").expect("Failed to create UserId.")
            ).expect("Failed to convert UserId to JSON."),
            r#""@carl:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_user_id() {
        assert_eq!(
            from_str::<UserId>(
                r#""@carl:example.com""#
            ).expect("Failed to convert JSON to UserId"),
            UserId::try_from("@carl:example.com").expect("Failed to create UserId.")
        );
    }

    #[test]
    fn valid_user_id_with_explicit_standard_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:443")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com"
        );
    }

    #[test]
    fn valid_user_id_with_non_standard_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:5000")
                .expect("Failed to create UserId.")
                .to_string(),
            "@carl:example.com:5000"
        );
    }

    #[test]
    fn invalid_characters_in_user_id_localpart() {
        assert_eq!(
            UserId::try_from("@%%%:example.com").err().unwrap(),
            Error::InvalidCharacters
        );
    }

    #[test]
    fn missing_user_id_sigil() {
        assert_eq!(
            UserId::try_from("carl:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_user_id_delimiter() {
        assert_eq!(
            UserId::try_from("@carl").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_user_id_host() {
        assert_eq!(
            UserId::try_from("@carl:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }
}
