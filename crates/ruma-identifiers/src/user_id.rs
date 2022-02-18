//! Matrix user identifiers.

use std::{rc::Rc, sync::Arc};

use crate::{MatrixToUri, ServerName};

/// A Matrix user ID.
///
/// A `UserId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::UserId;
/// assert_eq!(<&UserId>::try_from("@carl:example.com").unwrap(), "@carl:example.com");
/// ```
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(str);

opaque_identifier_validated!(UserId, ruma_identifiers_validation::user_id::validate);

impl UserId {
    /// Attempts to generate a `UserId` for the given origin server with a localpart consisting of
    /// 12 random ASCII characters.
    #[cfg(feature = "rand")]
    pub fn new(server_name: &ServerName) -> Box<Self> {
        Self::from_owned(
            format!("@{}:{}", crate::generate_localpart(12).to_lowercase(), server_name).into(),
        )
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
    ) -> Result<Box<Self>, crate::Error> {
        let id_str = id.as_ref();

        if id_str.starts_with('@') {
            Self::parse(id)
        } else {
            let _ = localpart_is_fully_conforming(id_str)?;
            Ok(Self::from_owned(format!("@{}:{}", id_str, server_name).into()))
        }
    }

    /// Variation of [`parse_with_server_name`] that returns `Rc<Self>`.
    ///
    /// [`parse_with_server_name`]: Self::parse_with_server_name
    pub fn parse_with_server_name_rc(
        id: impl AsRef<str> + Into<Rc<str>>,
        server_name: &ServerName,
    ) -> Result<Rc<Self>, crate::Error> {
        let id_str = id.as_ref();

        if id_str.starts_with('@') {
            Self::parse_rc(id)
        } else {
            let _ = localpart_is_fully_conforming(id_str)?;
            Ok(Self::from_rc(format!("@{}:{}", id_str, server_name).into()))
        }
    }

    /// Variation of [`parse_with_server_name`] that returns `Arc<Self>`.
    ///
    /// [`parse_with_server_name`]: Self::parse_with_server_name
    pub fn parse_with_server_name_arc(
        id: impl AsRef<str> + Into<Arc<str>>,
        server_name: &ServerName,
    ) -> Result<Arc<Self>, crate::Error> {
        let id_str = id.as_ref();

        if id_str.starts_with('@') {
            Self::parse_arc(id)
        } else {
            let _ = localpart_is_fully_conforming(id_str)?;
            Ok(Self::from_arc(format!("@{}:{}", id_str, server_name).into()))
        }
    }

    /// Returns the user's localpart.
    pub fn localpart(&self) -> &str {
        &self.as_str()[1..self.colon_idx() as usize]
    }

    /// Returns the server name of the user ID.
    pub fn server_name(&self) -> &ServerName {
        ServerName::from_borrowed(&self.as_str()[self.colon_idx() + 1..])
    }

    /// Whether this user ID is a historical one.
    ///
    /// A historical user ID is one that doesn't conform to the latest specification of the user ID
    /// grammar but is still accepted because it was previously allowed.
    pub fn is_historical(&self) -> bool {
        !localpart_is_fully_conforming(self.localpart()).unwrap()
    }

    /// Create a `matrix.to` reference for this user ID.
    ///
    /// # Example
    ///
    /// ```
    /// use ruma_identifiers::user_id;
    ///
    /// let message = format!(
    ///     r#"Thanks for the update <a href="{link}">{display_name}</a>."#,
    ///     link = user_id!("@jplatte:notareal.hs").matrix_to_url(),
    ///     display_name = "jplatte",
    /// );
    /// ```
    pub fn matrix_to_url(&self) -> MatrixToUri {
        MatrixToUri::new(self.into(), Vec::new())
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

pub use ruma_identifiers_validation::user_id::localpart_is_fully_conforming;

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::UserId;
    use crate::{server_name, Error};

    #[test]
    fn valid_user_id_from_str() {
        let user_id = <&UserId>::try_from("@carl:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn parse_valid_user_id() {
        let server_name = server_name!("example.com");
        let user_id = UserId::parse_with_server_name("@carl:example.com", server_name)
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn parse_valid_user_id_parts() {
        let server_name = server_name!("example.com");
        let user_id =
            UserId::parse_with_server_name("carl", server_name).expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@carl:example.com");
        assert_eq!(user_id.localpart(), "carl");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(!user_id.is_historical());
    }

    #[test]
    fn invalid_user_id() {
        let localpart = "τ";
        let user_id = "@τ:example.com";
        let server_name = server_name!("example.com");

        assert!(<&UserId>::try_from(user_id).is_err());
        assert!(UserId::parse_with_server_name(user_id, server_name).is_err());
        assert!(UserId::parse_with_server_name(localpart, server_name).is_err());
        assert!(UserId::parse_with_server_name_rc(user_id, server_name).is_err());
        assert!(UserId::parse_with_server_name_rc(localpart, server_name).is_err());
        assert!(UserId::parse_with_server_name_arc(user_id, server_name).is_err());
        assert!(UserId::parse_with_server_name_arc(localpart, server_name).is_err());
        assert!(UserId::parse_rc(user_id).is_err());
        assert!(UserId::parse_arc(user_id).is_err());
    }

    #[test]
    fn valid_historical_user_id() {
        let user_id =
            <&UserId>::try_from("@a%b[irc]:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn parse_valid_historical_user_id() {
        let server_name = server_name!("example.com");
        let user_id = UserId::parse_with_server_name("@a%b[irc]:example.com", server_name)
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn parse_valid_historical_user_id_parts() {
        let server_name = server_name!("example.com");
        let user_id = UserId::parse_with_server_name("a%b[irc]", server_name)
            .expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@a%b[irc]:example.com");
        assert_eq!(user_id.localpart(), "a%b[irc]");
        assert_eq!(user_id.server_name(), "example.com");
        assert!(user_id.is_historical());
    }

    #[test]
    fn uppercase_user_id() {
        let user_id = <&UserId>::try_from("@CARL:example.com").expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@CARL:example.com");
        assert!(user_id.is_historical());
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_user_id() {
        let server_name = server_name!("example.com");
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
            serde_json::to_string(
                <&UserId>::try_from("@carl:example.com").expect("Failed to create UserId.")
            )
            .expect("Failed to convert UserId to JSON."),
            r#""@carl:example.com""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_user_id() {
        assert_eq!(
            serde_json::from_str::<Box<UserId>>(r#""@carl:example.com""#)
                .expect("Failed to convert JSON to UserId"),
            <&UserId>::try_from("@carl:example.com").expect("Failed to create UserId.")
        );
    }

    #[test]
    fn valid_user_id_with_explicit_standard_port() {
        assert_eq!(
            <&UserId>::try_from("@carl:example.com:443")
                .expect("Failed to create UserId.")
                .as_ref(),
            "@carl:example.com:443"
        );
    }

    #[test]
    fn valid_user_id_with_non_standard_port() {
        let user_id =
            <&UserId>::try_from("@carl:example.com:5000").expect("Failed to create UserId.");
        assert_eq!(user_id.as_str(), "@carl:example.com:5000");
        assert!(!user_id.is_historical());
    }

    #[test]
    #[cfg(not(feature = "compat"))]
    fn invalid_characters_in_user_id_localpart() {
        assert_eq!(
            <&UserId>::try_from("@te\nst:example.com").unwrap_err(),
            Error::InvalidCharacters
        );
    }

    #[test]
    fn missing_user_id_sigil() {
        assert_eq!(
            <&UserId>::try_from("carl:example.com").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_user_id_delimiter() {
        assert_eq!(<&UserId>::try_from("@carl").unwrap_err(), Error::MissingDelimiter);
    }

    #[test]
    fn invalid_user_id_host() {
        assert_eq!(<&UserId>::try_from("@carl:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_user_id_port() {
        assert_eq!(
            <&UserId>::try_from("@carl:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
