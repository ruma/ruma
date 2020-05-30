//! Matrix user identifiers.

use std::{borrow::Cow, convert::TryFrom, num::NonZeroU8};

use crate::{error::Error, is_valid_server_name, parse_id};

/// A Matrix user ID.
///
/// A `UserId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::UserId;
/// assert_eq!(
///     UserId::try_from("@carl:example.com").unwrap().as_ref(),
///     "@carl:example.com"
/// );
/// ```
#[derive(Clone, Debug)]
pub struct UserId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
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
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new(server_name: &str) -> Result<Self, Error> {
        use crate::generate_localpart;

        if !is_valid_server_name(server_name) {
            return Err(Error::InvalidServerName);
        }
        let full_id = format!("@{}:{}", generate_localpart(12).to_lowercase(), server_name).into();

        Ok(Self {
            full_id,
            colon_idx: NonZeroU8::new(13).unwrap(),
            is_historical: false,
        })
    }

    /// Attempts to complete a user ID, by adding the colon + server name and `@` prefix, if not
    /// present already.
    ///
    /// This is a convenience function for the login API, where a user can supply either their full
    /// user ID or just the localpart. It only supports a valid user ID or a valid user ID
    /// localpart, not the localpart plus the `@` prefix, or the localpart plus server name without
    /// the `@` prefix.
    pub fn parse_with_server_name(
        id: impl AsRef<str> + Into<String>,
        server_name: &str,
    ) -> Result<Self, Error> {
        let id_str = id.as_ref();

        if id_str.starts_with('@') {
            Self::try_from(id.into())
        } else {
            let is_fully_conforming = localpart_is_fully_comforming(id_str)?;
            if !is_valid_server_name(server_name) {
                return Err(Error::InvalidServerName);
            }

            Ok(Self {
                full_id: format!("@{}:{}", id_str, server_name).into(),
                colon_idx: NonZeroU8::new(id_str.len() as u8 + 1).unwrap(),
                is_historical: !is_fully_conforming,
            })
        }
    }

    /// Returns the user's localpart.
    pub fn localpart(&self) -> &str {
        &self.full_id[1..self.colon_idx.get() as usize]
    }

    /// Returns the server name of the user ID.
    pub fn server_name(&self) -> &str {
        &self.full_id[self.colon_idx.get() as usize + 1..]
    }

    /// Whether this user ID is a historical one, i.e. one that doesn't conform to the latest
    /// specification of the user ID grammar but is still accepted because it was previously
    /// allowed.
    pub fn is_historical(&self) -> bool {
        self.is_historical
    }
}

impl TryFrom<Cow<'_, str>> for UserId {
    type Error = Error;

    /// Attempts to create a new Matrix user ID from a string representation.
    ///
    /// The string must include the leading @ sigil, the localpart, a literal colon, and a server
    /// name.
    fn try_from(user_id: Cow<'_, str>) -> Result<Self, Error> {
        let colon_idx = parse_id(&user_id, &['@'])?;
        let localpart = &user_id[1..colon_idx.get() as usize];

        let is_historical = localpart_is_fully_comforming(localpart)?;

        Ok(Self {
            full_id: user_id.into_owned().into(),
            colon_idx,
            is_historical: !is_historical,
        })
    }
}

common_impls!(UserId, "a Matrix user ID");

/// Check whether the given user id localpart is valid and fully conforming
///
/// Returns an `Err` for invalid user ID localparts, `Ok(false)` for historical user ID localparts
/// and `Ok(true)` for fully conforming user ID localparts.
pub fn localpart_is_fully_comforming(localpart: &str) -> Result<bool, Error> {
    if localpart.is_empty() {
        return Err(Error::InvalidLocalPart);
    }

    // See https://matrix.org/docs/spec/appendices#user-identifiers
    let is_fully_conforming = localpart.bytes().all(|b| match b {
        b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.' | b'=' | b'_' | b'/' => true,
        _ => false,
    });

    // If it's not fully conforming, check if it contains characters that are also disallowed
    // for historical user IDs. If there are, return an error.
    // See https://matrix.org/docs/spec/appendices#historical-user-ids
    if !is_fully_conforming && localpart.bytes().any(|b| b < 0x21 || b == b':' || b > 0x7E) {
        Err(Error::InvalidCharacters)
    } else {
        Ok(is_fully_conforming)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    use super::UserId;
    use crate::error::Error;

    #[test]
    fn valid_user_id_from_str() {
        let user_id = UserId::try_from("@carl:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn parse_valid_user_id() {
        let user_id = UserId::parse_with_server_name("@carl:example.com", "example.com")
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn parse_valid_user_id_parts() {
        let user_id = UserId::parse_with_server_name("carl", "example.com")
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn valid_historical_user_id() {
        let user_id = UserId::try_from("@a%b[irc]:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn parse_valid_historical_user_id() {
        let user_id = UserId::parse_with_server_name("@a%b[irc]:example.com", "example.com")
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn parse_valid_historical_user_id_parts() {
        let user_id = UserId::parse_with_server_name("a%b[irc]", "example.com")
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn uppercase_user_id() {
        let user_id = UserId::try_from("@CARL:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@CARL:example.com");
        assert!(user_id.is_historical());
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_user_id() {
        let user_id = UserId::new("example.com").expect("Failed to generate UserId.");
        assert_eq!(user_id.localpart().len(), 12);
        assert_eq!(user_id.server_name(), "example.com");

        let id_str: &str = user_id.as_ref();

        assert!(id_str.starts_with('@'));
        assert_eq!(id_str.len(), 25);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_invalid_user_id() {
        assert!(UserId::new("").is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_valid_user_id() {
        assert_eq!(
            to_string(&UserId::try_from("@carl:example.com").expect("Failed to create UserId."))
                .expect("Failed to convert UserId to JSON."),
            r#""@carl:example.com""#
        );
    }

    #[cfg(feature = "serde")]
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
                .as_ref(),
            "@carl:example.com:443"
        );
    }

    #[test]
    fn valid_user_id_with_non_standard_port() {
        let user_id = UserId::try_from("@carl:example.com:5000").expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@carl:example.com:5000");
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
    fn missing_localpart() {
        assert_eq!(
            UserId::try_from("@:example.com").unwrap_err(),
            Error::InvalidLocalPart
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
        assert_eq!(
            UserId::try_from("@carl:/").unwrap_err(),
            Error::InvalidServerName
        );
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
