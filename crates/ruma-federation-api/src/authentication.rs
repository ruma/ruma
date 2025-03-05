//! Common types for implementing federation authorization.

use std::{fmt, str::FromStr};

use headers::authorization::Credentials;
use http::HeaderValue;
use http_auth::ChallengeParser;
use ruma_common::{
    http_headers::quote_ascii_string_if_required,
    serde::{Base64, Base64DecodeError},
    IdParseError, OwnedServerName, OwnedServerSigningKeyId,
};
use thiserror::Error;
use tracing::debug;

/// Typed representation of an `Authorization` header of scheme `X-Matrix`, as defined in the
/// [Matrix Server-Server API][spec].
///
/// [spec]: https://spec.matrix.org/latest/server-server-api/#request-authentication
#[derive(Clone)]
#[non_exhaustive]
pub struct XMatrix {
    /// The server name of the sending server.
    pub origin: OwnedServerName,
    /// The server name of the receiving sender.
    ///
    /// For compatibility with older servers, recipients should accept requests without this
    /// parameter, but MUST always send it. If this property is included, but the value does
    /// not match the receiving server's name, the receiving server must deny the request with
    /// an HTTP status code 401 Unauthorized.
    pub destination: Option<OwnedServerName>,
    /// The ID - including the algorithm name - of the sending server's key that was used to sign
    /// the request.
    pub key: OwnedServerSigningKeyId,
    /// The signature of the JSON.
    pub sig: Base64,
}

impl XMatrix {
    /// Construct a new X-Matrix Authorization header.
    pub fn new(
        origin: OwnedServerName,
        destination: OwnedServerName,
        key: OwnedServerSigningKeyId,
        sig: Base64,
    ) -> Self {
        Self { origin, destination: Some(destination), key, sig }
    }

    /// Parse an X-Matrix Authorization header from the given string.
    pub fn parse(s: impl AsRef<str>) -> Result<Self, XMatrixParseError> {
        let parser = ChallengeParser::new(s.as_ref());
        let mut xmatrix = None;

        for challenge in parser {
            let challenge = challenge?;

            if challenge.scheme.eq_ignore_ascii_case(XMatrix::SCHEME) {
                xmatrix = Some(challenge);
                break;
            }
        }

        let Some(xmatrix) = xmatrix else {
            return Err(XMatrixParseError::NotFound);
        };

        let mut origin = None;
        let mut destination = None;
        let mut key = None;
        let mut sig = None;

        for (name, value) in xmatrix.params {
            if name.eq_ignore_ascii_case("origin") {
                if origin.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("origin".to_owned()));
                } else {
                    origin = Some(OwnedServerName::try_from(value.to_unescaped())?);
                }
            } else if name.eq_ignore_ascii_case("destination") {
                if destination.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("destination".to_owned()));
                } else {
                    destination = Some(OwnedServerName::try_from(value.to_unescaped())?);
                }
            } else if name.eq_ignore_ascii_case("key") {
                if key.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("key".to_owned()));
                } else {
                    key = Some(OwnedServerSigningKeyId::try_from(value.to_unescaped())?);
                }
            } else if name.eq_ignore_ascii_case("sig") {
                if sig.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("sig".to_owned()));
                } else {
                    sig = Some(Base64::parse(value.to_unescaped())?);
                }
            } else {
                debug!("Unknown parameter {name} in X-Matrix Authorization header");
            }
        }

        Ok(Self {
            origin: origin
                .ok_or_else(|| XMatrixParseError::MissingParameter("origin".to_owned()))?,
            destination,
            key: key.ok_or_else(|| XMatrixParseError::MissingParameter("key".to_owned()))?,
            sig: sig.ok_or_else(|| XMatrixParseError::MissingParameter("sig".to_owned()))?,
        })
    }
}

impl fmt::Debug for XMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XMatrix")
            .field("origin", &self.origin)
            .field("destination", &self.destination)
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for XMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { origin, destination, key, sig } = self;

        let origin = quote_ascii_string_if_required(origin.as_str());
        let key = quote_ascii_string_if_required(key.as_str());
        let sig = sig.encode();
        let sig = quote_ascii_string_if_required(&sig);

        write!(f, r#"{} "#, Self::SCHEME)?;

        if let Some(destination) = destination {
            let destination = quote_ascii_string_if_required(destination.as_str());
            write!(f, r#"destination={destination},"#)?;
        }

        write!(f, "key={key},origin={origin},sig={sig}")
    }
}

impl FromStr for XMatrix {
    type Err = XMatrixParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&HeaderValue> for XMatrix {
    type Error = XMatrixParseError;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        Self::parse(value.to_str()?)
    }
}

impl From<&XMatrix> for HeaderValue {
    fn from(value: &XMatrix) -> Self {
        value.to_string().try_into().expect("header format is static")
    }
}

impl Credentials for XMatrix {
    const SCHEME: &'static str = "X-Matrix";

    fn decode(value: &HeaderValue) -> Option<Self> {
        value.try_into().ok()
    }

    fn encode(&self) -> HeaderValue {
        self.into()
    }
}

/// An error when trying to parse an X-Matrix Authorization header.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XMatrixParseError {
    /// The `HeaderValue` could not be converted to a `str`.
    #[error(transparent)]
    ToStr(#[from] http::header::ToStrError),

    /// The string could not be parsed as a valid Authorization string.
    #[error("{0}")]
    ParseStr(String),

    /// The credentials with the X-Matrix scheme were not found.
    #[error("X-Matrix credentials not found")]
    NotFound,

    /// The parameter value could not be parsed as a Matrix ID.
    #[error(transparent)]
    ParseId(#[from] IdParseError),

    /// The parameter value could not be parsed as base64.
    #[error(transparent)]
    ParseBase64(#[from] Base64DecodeError),

    /// The parameter with the given name was not found.
    #[error("missing parameter '{0}'")]
    MissingParameter(String),

    /// The parameter with the given name was found more than once.
    #[error("duplicate parameter '{0}'")]
    DuplicateParameter(String),
}

impl<'a> From<http_auth::parser::Error<'a>> for XMatrixParseError {
    fn from(value: http_auth::parser::Error<'a>) -> Self {
        Self::ParseStr(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use headers::{authorization::Credentials, HeaderValue};
    use ruma_common::{serde::Base64, OwnedServerName};

    use super::XMatrix;

    #[test]
    fn xmatrix_auth_pre_1_3() {
        let header = HeaderValue::from_static(
            "X-Matrix origin=\"origin.hs.example.com\",key=\"ed25519:key1\",sig=\"dGVzdA==\"",
        );
        let origin = "origin.hs.example.com".try_into().unwrap();
        let key = "ed25519:key1".try_into().unwrap();
        let sig = Base64::new(b"test".to_vec());
        let credentials = XMatrix::try_from(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, None);
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix { origin, destination: None, key, sig };

        assert_eq!(
            credentials.encode(),
            "X-Matrix key=\"ed25519:key1\",origin=origin.hs.example.com,sig=dGVzdA"
        );
    }

    #[test]
    fn xmatrix_auth_1_3() {
        let header = HeaderValue::from_static("X-Matrix origin=\"origin.hs.example.com\",destination=\"destination.hs.example.com\",key=\"ed25519:key1\",sig=\"dGVzdA==\"");
        let origin: OwnedServerName = "origin.hs.example.com".try_into().unwrap();
        let destination: OwnedServerName = "destination.hs.example.com".try_into().unwrap();
        let key = "ed25519:key1".try_into().unwrap();
        let sig = Base64::new(b"test".to_vec());
        let credentials = XMatrix::try_from(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, Some(destination.clone()));
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix::new(origin, destination, key, sig);

        assert_eq!(credentials.encode(), "X-Matrix destination=destination.hs.example.com,key=\"ed25519:key1\",origin=origin.hs.example.com,sig=dGVzdA");
    }

    #[test]
    fn xmatrix_quoting() {
        let header = HeaderValue::from_static(
            r#"X-Matrix origin="example.com:1234",key="abc\"def\\:ghi",sig=dGVzdA,"#,
        );

        let origin: OwnedServerName = "example.com:1234".try_into().unwrap();
        let key = r#"abc"def\:ghi"#.try_into().unwrap();
        let sig = Base64::new(b"test".to_vec());
        let credentials = XMatrix::try_from(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, None);
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix { origin, destination: None, key, sig };

        assert_eq!(
            credentials.encode(),
            r#"X-Matrix key="abc\"def\\:ghi",origin="example.com:1234",sig=dGVzdA"#
        );
    }

    #[test]
    fn xmatrix_auth_1_3_with_extra_spaces() {
        let header = HeaderValue::from_static("X-Matrix origin=\"origin.hs.example.com\"  ,     destination=\"destination.hs.example.com\",key=\"ed25519:key1\", sig=\"dGVzdA\"");
        let credentials = XMatrix::try_from(&header).unwrap();
        let sig = Base64::new(b"test".to_vec());

        assert_eq!(credentials.origin, "origin.hs.example.com");
        assert_eq!(credentials.destination.unwrap(), "destination.hs.example.com");
        assert_eq!(credentials.key, "ed25519:key1");
        assert_eq!(credentials.sig, sig);
    }
}
