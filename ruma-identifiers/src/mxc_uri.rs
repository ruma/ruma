//! Matrix-spec compliant mxc:// urls.

use std::{convert::TryFrom, fmt, str::FromStr};

use ruma_identifiers_validation::mxc_uri::validate;

use crate::{Error, ServerName, ServerNameBox};

/// Matrix-spec compliant mxc:// urls.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MxcUri {
    server_name: ServerNameBox,
    media_id: Box<str>,
}

impl MxcUri {
    /// Returns the media ID of this mxc://.
    pub fn media_id(&self) -> &str {
        &self.media_id
    }

    /// Returns the server name of this mxc://.
    pub fn server_name(&self) -> &ServerName {
        &self.server_name
    }

    fn mxc_uri_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mxc://{}/{}", self.server_name, self.media_id))
    }
}

impl fmt::Debug for MxcUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.mxc_uri_fmt(f)
    }
}

impl fmt::Display for MxcUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.mxc_uri_fmt(f)
    }
}

fn try_from<S>(uri: S) -> Result<MxcUri, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let (media_id, server_name) = validate(uri.as_ref())?;
    Ok(MxcUri { media_id: media_id.into(), server_name: <ServerNameBox>::try_from(server_name)? })
}

impl FromStr for MxcUri {
    type Err = crate::Error;

    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        try_from(uri)
    }
}

impl TryFrom<&str> for MxcUri {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl TryFrom<String> for MxcUri {
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MxcUri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::deserialize_id(
            deserializer,
            "Content location represented as a Matrix Content (MXC) URI",
        )
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

    use super::MxcUri;

    #[test]
    fn parse_mxc_uri() {
        assert!(<MxcUri>::try_from("mxc://127.0.0.1/asd32asdfasdsd").is_ok());
    }

    #[test]
    fn parse_mxc_uri_without_media_id() {
        assert!(!<MxcUri>::try_from("mxc://127.0.0.1").is_ok());
    }

    #[test]
    fn parse_mxc_uri_without_protocol() {
        assert!(!<MxcUri>::try_from("127.0.0.1/asd32asdfasdsd").is_ok());
    }
}
