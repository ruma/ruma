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

lazy_static! {
    static ref  USER_ID_PATTERN: Regex =
        Regex::new(r"\A@(?P<localpart>[a-z0-9._=-]+):(?P<host>.+)\z")
        .expect("Failed to compile user ID regex.");
}

/// An error encountered when trying to parse an invalid user ID string.
#[derive(Debug)]
pub enum Error {
    /// The user ID string did not match the "@<localpart>:<domain>" format, or used invalid
    /// characters in its localpart.
    InvalidFormat,
    /// The domain part of the user ID string was not a valid IP address or DNS name.
    InvalidHost(ParseError),
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

impl UserId {
    /// Create a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a valid
    /// server name.
    pub fn new(user_id: &str) -> Result<UserId, Error> {
        let captures = match USER_ID_PATTERN.captures(user_id) {
            Some(captures) => captures,
            None => return Err(Error::InvalidFormat),
        };

        let raw_host = captures.name("host").expect("Failed to extract hostname from regex.");

        let url_string = format!("https://{}", raw_host);

        let url = try!(Url::parse(&url_string));

        let host = match url.host() {
            Some(host) => host,
            None => return Err(Error::InvalidFormat),
        };

        let port = url.port().unwrap_or(443);

        Ok(UserId {
            hostname: host.to_owned(),
            port: port,
            localpart: captures
                .name("localpart")
                .expect("Failed to extract localpart from regex.")
                .to_string(),
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
    fn from(error: ParseError) -> Error {
        Error::InvalidHost(error)
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
    use super::UserId;

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
        assert!(UserId::new("@CARL:example.com").is_err());
    }

    #[test]
    fn missing_sigil() {
        assert!(UserId::new("carl:example.com").is_err());
    }

    #[test]
    fn missing_domain() {
        assert!(UserId::new("carl").is_err());
    }

    #[test]
    fn invalid_host() {
        assert!(UserId::new("@carl:-").is_err());
    }

    #[test]
    fn invalid_port() {
        assert!(UserId::new("@carl:example.com:notaport").is_err());
    }
}
