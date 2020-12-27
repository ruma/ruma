//! Matrix user identifiers.

use std::{convert::TryFrom, fmt, num::NonZeroU8};

use crate::{Error, ServerName};

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
#[derive(Clone)]
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

impl fmt::Debug for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_id)
    }
}

impl UserId {
    /// Attempts to generate a `UserId` for the given origin server with a localpart consisting of
    /// 12 random ASCII characters.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new(server_name: &ServerName) -> Self {
        use crate::generate_localpart;

        let full_id = format!("@{}:{}", generate_localpart(12).to_lowercase(), server_name).into();

        Self { full_id, colon_idx: NonZeroU8::new(13).unwrap(), is_historical: false }
    }

    /// Attempts to complete a user ID, by adding the colon + server name and `@` prefix, if not
    /// present already.
    ///
    /// This is a convenience function for the login API, where a user can supply either their full
    /// user ID or just the localpart. It only supports a valid user ID or a valid user ID
    /// localpart, not the localpart plus the `@` prefix, or the localpart plus server name without
    /// the `@` prefix.
    pub fn parse_with_server_name(
        id: impl AsRef<str> + Into<Box<str>>,
        server_name: &ServerName,
    ) -> Result<Self, Error> {
        let id_str = id.as_ref();

        if id_str.starts_with('@') {
            try_from(id.into())
        } else {
            let is_fully_conforming = localpart_is_fully_comforming(id_str)?;

            Ok(Self {
                full_id: format!("@{}:{}", id_str, server_name).into(),
                colon_idx: NonZeroU8::new(id_str.len() as u8 + 1).unwrap(),
                is_historical: !is_fully_conforming,
            })
        }
    }
}

impl UserId {
    /// Returns the user's localpart.
    pub fn localpart(&self) -> &str {
        &self.full_id[1..self.colon_idx.get() as usize]
    }

    /// Returns the server name of the user ID.
    pub fn server_name(&self) -> &ServerName {
        <&ServerName>::try_from(&self.full_id[self.colon_idx.get() as usize + 1..]).unwrap()
    }

    /// Whether this user ID is a historical one, i.e. one that doesn't conform to the latest
    /// specification of the user ID grammar but is still accepted because it was previously
    /// allowed.
    pub fn is_historical(&self) -> bool {
        self.is_historical
    }
}

/// Attempts to create a new Matrix user ID from a string representation.
///
/// The string must include the leading @ sigil, the localpart, a literal colon, and a server name.
fn try_from<S>(user_id: S) -> Result<UserId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let (colon_idx, is_historical) =
        ruma_identifiers_validation::user_id::validate(user_id.as_ref())?;
    Ok(UserId { full_id: user_id.into(), colon_idx, is_historical: !is_historical })
}

common_impls!(UserId, try_from, "a Matrix user ID");

pub use ruma_identifiers_validation::user_id::localpart_is_fully_comforming;

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    use super::UserId;
    use crate::{Error, ServerName};

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
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        let user_id = UserId::parse_with_server_name("@carl:example.com", server_name)
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn parse_valid_user_id_parts() {
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        let user_id =
            UserId::parse_with_server_name("carl", server_name).expect("Failed to create UserId.");
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
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        let user_id = UserId::parse_with_server_name("@a%b[irc]:example.com", server_name)
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_ref(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn parse_valid_historical_user_id_parts() {
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        let user_id = UserId::parse_with_server_name("a%b[irc]", server_name)
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
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        let user_id = UserId::new(server_name);
        assert_eq!(user_id.localpart().len(), 12);
        assert_eq!(user_id.server_name(), "example.com");

        let id_str = user_id.as_str();

        assert!(id_str.starts_with('@'));
        assert_eq!(id_str.len(), 25);
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
            UserId::try_from("@carl:example.com:443").expect("Failed to create UserId.").as_ref(),
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
        assert_eq!(UserId::try_from("@te\nst:example.com").unwrap_err(), Error::InvalidCharacters);
    }

    #[test]
    fn missing_user_id_sigil() {
        assert_eq!(UserId::try_from("carl:example.com").unwrap_err(), Error::MissingLeadingSigil);
    }

    #[test]
    fn missing_user_id_delimiter() {
        assert_eq!(UserId::try_from("@carl").unwrap_err(), Error::MissingDelimiter);
    }

    #[test]
    fn invalid_user_id_host() {
        assert_eq!(UserId::try_from("@carl:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            UserId::try_from("@carl:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
