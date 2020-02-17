//! Matrix user identifiers.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Host;

use crate::{deserialize_id, display, error::Error, generate_localpart, parse_id};

/// A Matrix user ID.
///
/// A `UserId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::UserId;
/// assert_eq!(
///     UserId::try_from("@carl:example.com").unwrap().to_string(),
///     "@carl:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct UserId {
    /// The hostname of the homeserver.
    hostname: Host,
    /// The user's unique ID.
    localpart: String,
    /// The network port of the homeserver.
    port: u16,
    /// Whether this user id is a historical one.
    ///
    /// A historical user id is one that is not legal per the regular user id rules, but was
    /// accepted by previous versions of the spec and thus has to be supported because users with
    /// these kinds of ids still exist.
    is_historical: bool,
}

impl UserId {
    /// Attempts to generate a `UserId` for the given origin server with a localpart consisting of
    /// 12 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    pub fn new(homeserver_host: &str) -> Result<Self, Error> {
        let user_id = format!(
            "@{}:{}",
            generate_localpart(12).to_lowercase(),
            homeserver_host
        );
        let (localpart, host, port) = parse_id('@', &user_id)?;

        Ok(Self {
            hostname: host,
            localpart: localpart.to_string(),
            port,
            is_historical: false,
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

    /// Whether this user ID is a historical one, i.e. one that doesn't conform to the latest
    /// specification of the user ID grammar but is still accepted because it was previously
    /// allowed.
    pub fn is_historical(&self) -> bool {
        self.is_historical
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display(f, '@', &self.localpart, &self.hostname, self.port)
    }
}

impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_id(deserializer, "a Matrix user ID as a string")
    }
}

impl TryFrom<&str> for UserId {
    type Error = Error;

    /// Attempts to create a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a valid
    /// server name.
    fn try_from(user_id: &str) -> Result<Self, Error> {
        let (localpart, host, port) = parse_id('@', user_id)?;

        // See https://matrix.org/docs/spec/appendices#user-identifiers
        let is_fully_conforming = localpart.bytes().all(|b| match b {
            b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.' | b'=' | b'_' | b'/' => true,
            _ => false,
        });

        // If it's not fully conforming, check if it contains characters that are also disallowed
        // for historical user IDs. If there are, return an error.
        // See https://matrix.org/docs/spec/appendices#historical-user-ids
        if !is_fully_conforming && localpart.bytes().any(|b| b < 0x21 || b == b':' || b > 0x7E) {
            return Err(Error::InvalidCharacters);
        }

        Ok(Self {
            hostname: host,
            port,
            localpart: localpart.to_owned(),
            is_historical: !is_fully_conforming,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::UserId;
    use crate::error::Error;

    #[test]
    fn valid_user_id() {
        let user_id = UserId::try_from("@carl:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.to_string(), "@carl:example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn valid_historical_user_id() {
        let user_id = UserId::try_from("@a%b[irc]:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.to_string(), "@a%b[irc]:example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn downcase_user_id() {
        let user_id = UserId::try_from("@CARL:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.to_string(), "@CARL:example.com");
        assert!(user_id.is_historical());
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
            to_string(&UserId::try_from("@carl:example.com").expect("Failed to create UserId."))
                .expect("Failed to convert UserId to JSON."),
            r#""@carl:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_user_id() {
        assert_eq!(
            from_str::<UserId>(r#""@carl:example.com""#).expect("Failed to convert JSON to UserId"),
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
        let user_id = UserId::try_from("@carl:example.com:5000").expect("Failed to create UserId.");
        assert_eq!(user_id.to_string(), "@carl:example.com:5000");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn invalid_characters_in_user_id_localpart() {
        assert_eq!(
            UserId::try_from("@te\nst:example.com").unwrap_err(),
            Error::InvalidCharacters
        );
    }

    #[test]
    fn missing_user_id_sigil() {
        assert_eq!(
            UserId::try_from("carl:example.com").unwrap_err(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_user_id_delimiter() {
        assert_eq!(
            UserId::try_from("@carl").unwrap_err(),
            Error::MissingDelimiter
        );
    }

    #[test]
    fn invalid_user_id_host() {
        assert_eq!(UserId::try_from("@carl:/").unwrap_err(), Error::InvalidHost);
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:notaport").unwrap_err(),
            Error::InvalidHost
        );
    }
}
