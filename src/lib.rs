//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) opaque identifiers,
//! such as user IDs, room IDs, and room aliases.

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate url;

use std::fmt::{Display, Formatter, Result as FmtResult};

use regex::Regex;
use url::{Host, ParseError, Url};

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

/// An error encountered when trying to parse an invalid user ID string.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The ID's localpart contains invalid characters.
    InvalidCharacters,
    /// The domain part of the user ID string was not a valid IP address or DNS name.
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
    let url = try!(Url::parse(&url_string));

    let host = match url.host() {
        Some(host) => host.to_owned(),
        None => return Err(Error::InvalidHost),
    };

    let port = url.port().unwrap_or(443);

    Ok((localpart, host, port))
}

impl UserId {
    /// Create a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a valid
    /// server name.
    pub fn new(user_id: &str) -> Result<UserId, Error> {
        let (localpart, host, port) = try!(parse_id('@', user_id));

        if !USER_LOCALPART_PATTERN.is_match(localpart) {
            return Err(Error::InvalidCharacters);
        }

        Ok(UserId {
            hostname: host,
            port: port,
            localpart: localpart.to_owned(),
        })
    }

    /// Returns a `url::Host` for the user ID, containing the server name (minus the port) of the
    /// user's homeserver.
    ///
    /// This host can be either a domain name, an IPv4 address, or an IPv6 address.
    pub fn hostname(&self) -> &Host {
        &self.hostname
    }

    /// Returns the user's localpart.
    pub fn localpart(&self) -> &str {
        &self.localpart
    }

    /// Returns the port the user's homeserver can be accessed on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Error {
        Error::InvalidHost
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if self.port == 443 {
            write!(f, "@{}:{}", self.localpart, self.hostname)
        } else {
            write!(f, "@{}:{}:{}", self.localpart, self.hostname, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, UserId};

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
    fn invalid_characters_in_localpart() {
        assert_eq!(
            UserId::new("@CARL:example.com").err().unwrap(),
            Error::InvalidCharacters
        );
    }

    #[test]
    fn missing_sigil() {
        assert_eq!(
            UserId::new("carl:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            UserId::new("@carl").err().unwrap(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_host() {
        assert_eq!(
            UserId::new("@carl:-").err().unwrap(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_port() {
        assert_eq!(
            UserId::new("@carl:example.com:notaport").err().unwrap(),
            Error::InvalidHost
        );
    }
}
