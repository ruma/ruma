//! A URI that should be a Matrix-spec compliant MXC URI.

use std::{convert::TryFrom, fmt, num::NonZeroU8};

use ruma_identifiers_validation::mxc_uri::validate;

use crate::ServerName;

/// A URI that should be a Matrix-spec compliant MXC URI.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MxcUri {
    full_uri: Box<str>,
    slash_idx: Option<NonZeroU8>,
}

impl MxcUri {
    /// If this is a valid MXC URI, returns the media ID.
    pub fn media_id(&self) -> Option<&str> {
        self.slash_idx.map(|idx| &self.full_uri[idx.get() as usize + 1..])
    }

    /// If this is a valid MXC URI, returns the server name.
    pub fn server_name(&self) -> Option<&ServerName> {
        self.slash_idx
            .map(|idx| <&ServerName>::try_from(&self.full_uri[6..idx.get() as usize]).unwrap())
    }

    /// If this is a valid MXC URI, returns a `(server_name, media_id)` tuple.
    pub fn parts(&self) -> Option<(&ServerName, &str)> {
        self.slash_idx.map(|_| (self.server_name().unwrap(), self.media_id().unwrap()))
    }

    /// Returns if this is a spec-compliant MXC URI.
    pub fn is_valid(&self) -> bool {
        self.slash_idx.is_some()
    }

    /// Create a string slice from this MXC URI.
    pub fn as_str(&self) -> &str {
        &self.full_uri
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
    match validate(uri.as_ref()) {
        Ok(idx) => MxcUri { full_uri: uri.into(), slash_idx: Some(idx) },
        Err(_) => MxcUri { full_uri: uri.into(), slash_idx: None },
    }
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Into::into)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MxcUri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::ServerName;

    use super::MxcUri;

    #[test]
    fn parse_mxc_uri() {
        let mxc = MxcUri::from("mxc://127.0.0.1/asd32asdfasdsd");

        assert!(mxc.is_valid());
        assert_eq!(
            mxc.parts(),
            Some((
                <&ServerName>::try_from("127.0.0.1").expect("Failed to create ServerName"),
                "asd32asdfasdsd"
            ))
        );
    }

    #[test]
    fn parse_mxc_uri_without_media_id() {
        let mxc = MxcUri::from("mxc://127.0.0.1");

        assert!(!mxc.is_valid());
        assert_eq!(mxc.parts(), None);
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
            Some((
                <&ServerName>::try_from("server").expect("Failed to create ServerName"),
                "1234id"
            ))
        );
    }
}
