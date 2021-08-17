//! A URI that should be a Matrix-spec compliant [MXC URI].
//!
//! [MXC URI]: https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri

use std::{convert::TryFrom, fmt, num::NonZeroU8};

use ruma_identifiers_validation::{error::MxcUriError, mxc_uri::validate};

use crate::ServerName;

type Result<T> = std::result::Result<T, MxcUriError>;

/// A URI that should be a Matrix-spec compliant [MXC URI].
///
/// [MXC URI]: https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MxcUri {
    full_uri: Box<str>,
}

impl MxcUri {
    /// If this is a valid MXC URI, returns the media ID.
    pub fn media_id(&self) -> Result<&str> {
        self.parts().map(|(_, s)| s)
    }

    /// If this is a valid MXC URI, returns the server name.
    pub fn server_name(&self) -> Result<&ServerName> {
        self.parts().map(|(s, _)| s)
    }

    /// If this is a valid MXC URI, returns a `(server_name, media_id)` tuple, else it returns the
    /// error.
    pub fn parts(&self) -> Result<(&ServerName, &str)> {
        self.extract_slash_idx().map(|idx| {
            (
                <&ServerName>::try_from(&self.full_uri[6..idx.get() as usize]).unwrap(),
                &self.full_uri[idx.get() as usize + 1..],
            )
        })
    }

    /// Validates the URI and returns an error if it failed.
    pub fn validate(&self) -> Result<()> {
        self.extract_slash_idx().map(|_| ())
    }

    /// Convenience method for `.validate().is_ok()`.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Create a string slice from this MXC URI.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.full_uri
    }

    // convenience method for calling validate(self)
    #[inline(always)]
    fn extract_slash_idx(&self) -> Result<NonZeroU8> {
        validate(self.as_str())
    }
}

impl fmt::Debug for MxcUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_uri)
    }
}

impl fmt::Display for MxcUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_uri)
    }
}

fn from<S>(uri: S) -> MxcUri
where
    S: AsRef<str> + Into<Box<str>>,
{
    MxcUri { full_uri: uri.into() }
}

impl From<&str> for MxcUri {
    fn from(s: &str) -> Self {
        from(s)
    }
}

impl From<String> for MxcUri {
    fn from(s: String) -> Self {
        from(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MxcUri {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Into::into)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MxcUri {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use ruma_identifiers_validation::error::MxcUriError;

    use crate::ServerName;

    use super::MxcUri;

    #[test]
    fn parse_mxc_uri() {
        let mxc = MxcUri::from("mxc://127.0.0.1/asd32asdfasdsd");

        assert!(mxc.is_valid());
        assert_eq!(
            mxc.parts(),
            Ok((
                <&ServerName>::try_from("127.0.0.1").expect("Failed to create ServerName"),
                "asd32asdfasdsd"
            ))
        );
    }

    #[test]
    fn parse_mxc_uri_without_media_id() {
        let mxc = MxcUri::from("mxc://127.0.0.1");

        assert!(!mxc.is_valid());
        assert_eq!(mxc.parts(), Err(MxcUriError::MissingSlash));
    }

    #[test]
    fn parse_mxc_uri_without_protocol() {
        assert!(!MxcUri::from("127.0.0.1/asd32asdfasdsd").is_valid());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_mxc_uri() {
        assert_eq!(
            serde_json::to_string(&MxcUri::from("mxc://server/1234id"))
                .expect("Failed to convert MxcUri to JSON."),
            r#""mxc://server/1234id""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_mxc_uri() {
        let mxc = serde_json::from_str::<MxcUri>(r#""mxc://server/1234id""#)
            .expect("Failed to convert JSON to MxcUri");

        assert_eq!(mxc.as_str(), "mxc://server/1234id");
        assert!(mxc.is_valid());
        assert_eq!(
            mxc.parts(),
            Ok((<&ServerName>::try_from("server").expect("Failed to create ServerName"), "1234id"))
        );
    }
}
