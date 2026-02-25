//! Common types for implementing federation authorization.

use std::{fmt, str::FromStr};

use headers::authorization::Credentials;
use http::HeaderValue;
use http_auth::ChallengeParser;
use ruma_common::{
    CanonicalJsonObject, IdParseError, ServerName, ServerSigningKeyId,
    api::auth_scheme::AuthScheme,
    http_headers::quote_ascii_string_if_required,
    serde::{Base64, Base64DecodeError},
};
use ruma_signatures::{Ed25519KeyPair, KeyPair, PublicKeyMap};
use thiserror::Error;
use tracing::debug;

/// Authentication is performed by adding an `X-Matrix` header including a signature in the request
/// headers, as defined in the [Matrix Server-Server API][spec].
///
/// [spec]: https://spec.matrix.org/latest/server-server-api/#request-authentication
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::exhaustive_structs)]
pub struct ServerSignatures;

impl AuthScheme for ServerSignatures {
    type Input<'a> = ServerSignaturesInput<'a>;
    type AddAuthenticationError = XMatrixFromRequestError;
    type Output = XMatrix;
    type ExtractAuthenticationError = XMatrixExtractError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        input: ServerSignaturesInput<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        let authorization = HeaderValue::from(&XMatrix::try_from_http_request(request, input)?);
        request.headers_mut().insert(http::header::AUTHORIZATION, authorization);

        Ok(())
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<Self::Output, Self::ExtractAuthenticationError> {
        let value = request
            .headers()
            .get(http::header::AUTHORIZATION)
            .ok_or(XMatrixExtractError::MissingAuthorizationHeader)?;
        Ok(value.try_into()?)
    }
}

/// The input necessary to generate the [`ServerSignatures`] authentication scheme.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ServerSignaturesInput<'a> {
    /// The server making the request.
    pub origin: ServerName,

    /// The server receiving the request.
    pub destination: ServerName,

    /// The key pair to use to sign the request.
    pub key_pair: &'a Ed25519KeyPair,
}

impl<'a> ServerSignaturesInput<'a> {
    /// Construct a new `ServerSignaturesInput` with the given origin and key pair.
    pub fn new(origin: ServerName, destination: ServerName, key_pair: &'a Ed25519KeyPair) -> Self {
        Self { origin, destination, key_pair }
    }
}

/// Typed representation of an `Authorization` header of scheme `X-Matrix`, as defined in the
/// [Matrix Server-Server API][spec].
///
/// [spec]: https://spec.matrix.org/latest/server-server-api/#request-authentication
#[derive(Clone)]
#[non_exhaustive]
pub struct XMatrix {
    /// The server name of the sending server.
    pub origin: ServerName,
    /// The server name of the receiving sender.
    ///
    /// For compatibility with older servers, recipients should accept requests without this
    /// parameter, but MUST always send it. If this property is included, but the value does
    /// not match the receiving server's name, the receiving server must deny the request with
    /// an HTTP status code 401 Unauthorized.
    pub destination: Option<ServerName>,
    /// The ID - including the algorithm name - of the sending server's key that was used to sign
    /// the request.
    pub key: ServerSigningKeyId,
    /// The signature of the JSON.
    pub sig: Base64,
}

impl XMatrix {
    /// Construct a new X-Matrix Authorization header.
    pub fn new(
        origin: ServerName,
        destination: ServerName,
        key: ServerSigningKeyId,
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
                    origin = Some(ServerName::try_from(value.to_unescaped())?);
                }
            } else if name.eq_ignore_ascii_case("destination") {
                if destination.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("destination".to_owned()));
                } else {
                    destination = Some(ServerName::try_from(value.to_unescaped())?);
                }
            } else if name.eq_ignore_ascii_case("key") {
                if key.is_some() {
                    return Err(XMatrixParseError::DuplicateParameter("key".to_owned()));
                } else {
                    key = Some(ServerSigningKeyId::try_from(value.to_unescaped())?);
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

    /// Construct the canonical JSON object representation of the request to sign for the `XMatrix`
    /// scheme.
    pub fn request_object<T: AsRef<[u8]>>(
        request: &http::Request<T>,
        origin: &ServerName,
        destination: &ServerName,
    ) -> Result<CanonicalJsonObject, serde_json::Error> {
        let body = request.body().as_ref();
        let uri = request.uri().path_and_query().expect("http::Request should have a path");

        let mut request_object = CanonicalJsonObject::from([
            ("destination".to_owned(), destination.as_str().into()),
            ("method".to_owned(), request.method().as_str().into()),
            ("origin".to_owned(), origin.as_str().into()),
            ("uri".to_owned(), uri.as_str().into()),
        ]);

        if !body.is_empty() {
            let content = serde_json::from_slice(body)?;
            request_object.insert("content".to_owned(), content);
        }

        Ok(request_object)
    }

    /// Try to construct this header from the given HTTP request and input.
    pub fn try_from_http_request<T: AsRef<[u8]>>(
        request: &http::Request<T>,
        input: ServerSignaturesInput<'_>,
    ) -> Result<Self, XMatrixFromRequestError> {
        let ServerSignaturesInput { origin, destination, key_pair } = input;

        let request_object = Self::request_object(request, &origin, &destination)?;

        // The spec says to use the algorithm to sign JSON, so we could use
        // ruma_signatures::sign_json, however since we would need to extract the signature from the
        // JSON afterwards let's be a bit more efficient about it.
        let serialized_request_object = serde_json::to_vec(&request_object)?;
        let (key_id, signature) = key_pair.sign(&serialized_request_object).into_parts();

        let key = ServerSigningKeyId::try_from(key_id.as_str())
            .map_err(XMatrixFromRequestError::SigningKeyId)?;
        let sig = Base64::new(signature);

        Ok(Self { origin, destination: Some(destination), key, sig })
    }

    /// Verify that the signature in the `sig` field is valid for the given incoming HTTP request,
    /// with the given public keys map from the origin.
    pub fn verify_request<T: AsRef<[u8]>>(
        &self,
        request: &http::Request<T>,
        destination: &ServerName,
        public_key_map: &PublicKeyMap,
    ) -> Result<(), XMatrixVerificationError> {
        if self
            .destination
            .as_ref()
            .is_some_and(|xmatrix_destination| xmatrix_destination != destination)
        {
            return Err(XMatrixVerificationError::DestinationMismatch);
        }

        let mut request_object = Self::request_object(request, &self.origin, destination)
            .map_err(|error| ruma_signatures::Error::Json(error.into()))?;
        let entity_signature =
            CanonicalJsonObject::from([(self.key.to_string(), self.sig.encode().into())]);
        let signatures =
            CanonicalJsonObject::from([(self.origin.to_string(), entity_signature.into())]);
        request_object.insert("signatures".to_owned(), signatures.into());

        Ok(ruma_signatures::verify_json(public_key_map, &request_object)?)
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

/// An error when trying to construct an [`XMatrix`] from a [`http::Request`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XMatrixFromRequestError {
    /// Failed to construct the request object to sign.
    #[error("failed to construct request object to sign: {0}")]
    IntoJson(#[from] serde_json::Error),

    /// The signing key ID is invalid.
    #[error("invalid signing key ID: {0}")]
    SigningKeyId(IdParseError),
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

/// An error when trying to extract an [`XMatrix`] from an HTTP request.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XMatrixExtractError {
    /// No Authorization HTTP header was found, but the endpoint requires a server signature.
    #[error("no Authorization HTTP header found, but this endpoint requires a server signature")]
    MissingAuthorizationHeader,

    /// Failed to convert the header value to an [`XMatrix`].
    #[error("failed to parse header value: {0}")]
    Parse(#[from] XMatrixParseError),
}

/// An error when trying to verify the signature in an [`XMatrix`] for an HTTP request.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XMatrixVerificationError {
    /// The `destination` in [`XMatrix`] doesn't match the one to verify.
    #[error("destination in XMatrix doesn't match the one to verify")]
    DestinationMismatch,

    /// The signature verification failed.
    #[error("signature verification failed: {0}")]
    Signature(#[from] ruma_signatures::Error),
}

#[cfg(test)]
mod tests {
    use headers::{HeaderValue, authorization::Credentials};
    use ruma_common::{ServerName, serde::Base64};

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
        let header = HeaderValue::from_static(
            "X-Matrix origin=\"origin.hs.example.com\",destination=\"destination.hs.example.com\",key=\"ed25519:key1\",sig=\"dGVzdA==\"",
        );
        let origin: ServerName = "origin.hs.example.com".try_into().unwrap();
        let destination: ServerName = "destination.hs.example.com".try_into().unwrap();
        let key = "ed25519:key1".try_into().unwrap();
        let sig = Base64::new(b"test".to_vec());
        let credentials = XMatrix::try_from(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, Some(destination.clone()));
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix::new(origin, destination, key, sig);

        assert_eq!(
            credentials.encode(),
            "X-Matrix destination=destination.hs.example.com,key=\"ed25519:key1\",origin=origin.hs.example.com,sig=dGVzdA"
        );
    }

    #[test]
    fn xmatrix_quoting() {
        let header = HeaderValue::from_static(
            r#"X-Matrix origin="example.com:1234",key="abc\"def\\:ghi",sig=dGVzdA,"#,
        );

        let origin: ServerName = "example.com:1234".try_into().unwrap();
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
        let header = HeaderValue::from_static(
            "X-Matrix origin=\"origin.hs.example.com\"  ,     destination=\"destination.hs.example.com\",key=\"ed25519:key1\", sig=\"dGVzdA\"",
        );
        let credentials = XMatrix::try_from(&header).unwrap();
        let sig = Base64::new(b"test".to_vec());

        assert_eq!(credentials.origin, "origin.hs.example.com");
        assert_eq!(credentials.destination.unwrap(), "destination.hs.example.com");
        assert_eq!(credentials.key, "ed25519:key1");
        assert_eq!(credentials.sig, sig);
    }
}
