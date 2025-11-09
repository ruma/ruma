//! The `AuthScheme` trait used to specify the authentication scheme used by endpoints and the types
//! that implement it.

#![allow(clippy::exhaustive_structs)]

use as_variant::as_variant;
use http::{HeaderMap, header};
use serde::Deserialize;

/// Trait implemented by types representing an authentication scheme used by an endpoint.
pub trait AuthScheme: Sized {
    /// The input necessary to generate the authentication.
    type Input<'a>;

    /// The error type returned from [`add_authentication()`](Self::add_authentication).
    type AddAuthenticationError: Into<Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// The authentication data that can be extracted from a request.
    type Output;

    /// The error type returned from [`extract_authentication()`](Self::extract_authentication).
    type ExtractAuthenticationError: Into<Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// Add this authentication scheme to the given outgoing request, if necessary.
    ///
    /// Returns an error if the endpoint requires authentication but the input doesn't provide it,
    /// or if the input fails to serialize to the proper format.
    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        input: Self::Input<'_>,
    ) -> Result<(), Self::AddAuthenticationError>;

    /// Extract the data of this authentication scheme from the given incoming request.
    ///
    /// Returns an error if the endpoint requires authentication but the request doesn't provide it,
    /// or if the output fails to deserialize to the proper format.
    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<Self::Output, Self::ExtractAuthenticationError>;
}

/// No authentication is performed.
///
/// This type accepts a [`SendAccessToken`] as input to be able to send it regardless of whether it
/// is required.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAuthentication;

impl AuthScheme for NoAuthentication {
    type Input<'a> = SendAccessToken<'a>;
    type AddAuthenticationError = header::InvalidHeaderValue;
    type Output = ();
    type ExtractAuthenticationError = std::convert::Infallible;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        if let Some(access_token) = access_token.get_not_required_for_endpoint() {
            add_access_token_as_authorization_header(request.headers_mut(), access_token)?;
        }

        Ok(())
    }

    /// Since this endpoint doesn't expect any authentication, this is a noop.
    fn extract_authentication<T: AsRef<[u8]>>(
        _request: &http::Request<T>,
    ) -> Result<(), Self::ExtractAuthenticationError> {
        Ok(())
    }
}

/// Authentication is performed by including an access token in the `Authentication` http
/// header, or an `access_token` query parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessToken;

impl AuthScheme for AccessToken {
    type Input<'a> = SendAccessToken<'a>;
    type AddAuthenticationError = AddRequiredTokenError;
    /// The access token.
    type Output = String;
    type ExtractAuthenticationError = ExtractTokenError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        let token = access_token
            .get_required_for_endpoint()
            .ok_or(AddRequiredTokenError::MissingAccessToken)?;
        Ok(add_access_token_as_authorization_header(request.headers_mut(), token)?)
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<String, Self::ExtractAuthenticationError> {
        extract_bearer_or_query_token(request)?.ok_or(ExtractTokenError::MissingAccessToken)
    }
}

/// Authentication is optional, and it is performed by including an access token in the
/// `Authentication` http header, or an `access_token` query parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessTokenOptional;

impl AuthScheme for AccessTokenOptional {
    type Input<'a> = SendAccessToken<'a>;
    type AddAuthenticationError = header::InvalidHeaderValue;
    /// The access token, if any.
    type Output = Option<String>;
    type ExtractAuthenticationError = ExtractTokenError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        if let Some(access_token) = access_token.get_required_for_endpoint() {
            add_access_token_as_authorization_header(request.headers_mut(), access_token)?;
        }

        Ok(())
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<Option<String>, Self::ExtractAuthenticationError> {
        extract_bearer_or_query_token(request)
    }
}

/// Authentication is required, and can only be performed for appservices, by including an
/// appservice access token in the `Authentication` http header, or `access_token` query
/// parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppserviceToken;

impl AuthScheme for AppserviceToken {
    type Input<'a> = SendAccessToken<'a>;
    type AddAuthenticationError = AddRequiredTokenError;
    /// The appservice token.
    type Output = String;
    type ExtractAuthenticationError = ExtractTokenError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        let token = access_token
            .get_required_for_appservice()
            .ok_or(AddRequiredTokenError::MissingAccessToken)?;
        Ok(add_access_token_as_authorization_header(request.headers_mut(), token)?)
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<String, Self::ExtractAuthenticationError> {
        extract_bearer_or_query_token(request)?.ok_or(ExtractTokenError::MissingAccessToken)
    }
}

/// No authentication is performed for clients, but it can be performed for appservices, by
/// including an appservice access token in the `Authentication` http header, or an
/// `access_token` query parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppserviceTokenOptional;

impl AuthScheme for AppserviceTokenOptional {
    type Input<'a> = SendAccessToken<'a>;
    type AddAuthenticationError = header::InvalidHeaderValue;
    /// The appservice token, if any.
    type Output = Option<String>;
    type ExtractAuthenticationError = ExtractTokenError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), Self::AddAuthenticationError> {
        if let Some(access_token) = access_token.get_required_for_appservice() {
            add_access_token_as_authorization_header(request.headers_mut(), access_token)?;
        }

        Ok(())
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<Option<String>, Self::ExtractAuthenticationError> {
        extract_bearer_or_query_token(request)
    }
}

/// Add the given access token as an `Authorization` HTTP header to the given map.
fn add_access_token_as_authorization_header(
    headers: &mut HeaderMap,
    token: &str,
) -> Result<(), header::InvalidHeaderValue> {
    headers.insert(header::AUTHORIZATION, format!("Bearer {token}").try_into()?);
    Ok(())
}

/// Extract the access token from the `Authorization` HTTP header or the query string of the given
/// request.
fn extract_bearer_or_query_token<T>(
    request: &http::Request<T>,
) -> Result<Option<String>, ExtractTokenError> {
    if let Some(token) = extract_bearer_token_from_authorization_header(request.headers())? {
        return Ok(Some(token));
    }

    if let Some(query) = request.uri().query() {
        Ok(extract_access_token_from_query(query)?)
    } else {
        Ok(None)
    }
}

/// Extract the value of the `Authorization` HTTP header with a `Bearer` scheme.
fn extract_bearer_token_from_authorization_header(
    headers: &HeaderMap,
) -> Result<Option<String>, ExtractTokenError> {
    const EXPECTED_START: &str = "bearer ";

    let Some(value) = headers.get(header::AUTHORIZATION) else {
        return Ok(None);
    };

    let value = value.to_str()?;

    if value.len() < EXPECTED_START.len()
        || !value[..EXPECTED_START.len()].eq_ignore_ascii_case(EXPECTED_START)
    {
        return Err(ExtractTokenError::InvalidAuthorizationScheme);
    }

    Ok(Some(value[EXPECTED_START.len()..].to_owned()))
}

/// Extract the `access_token` from the given query string.
fn extract_access_token_from_query(
    query: &str,
) -> Result<Option<String>, serde_html_form::de::Error> {
    #[derive(Deserialize)]
    struct AccessTokenDeHelper {
        access_token: Option<String>,
    }

    serde_html_form::from_str::<AccessTokenDeHelper>(query).map(|helper| helper.access_token)
}

/// An enum to control whether an access token should be added to outgoing requests
#[derive(Clone, Copy, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SendAccessToken<'a> {
    /// Add the given access token to the request only if the `METADATA` on the request requires
    /// it.
    IfRequired(&'a str),

    /// Always add the access token.
    Always(&'a str),

    /// Add the given appservice token to the request only if the `METADATA` on the request
    /// requires it.
    Appservice(&'a str),

    /// Don't add an access token.
    ///
    /// This will lead to an error if the request endpoint requires authentication
    None,
}

impl<'a> SendAccessToken<'a> {
    /// Get the access token for an endpoint that requires one.
    ///
    /// Returns `Some(_)` if `self` contains an access token.
    pub fn get_required_for_endpoint(self) -> Option<&'a str> {
        as_variant!(self, Self::IfRequired | Self::Appservice | Self::Always)
    }

    /// Get the access token for an endpoint that should not require one.
    ///
    /// Returns `Some(_)` only if `self` is `SendAccessToken::Always(_)`.
    pub fn get_not_required_for_endpoint(self) -> Option<&'a str> {
        as_variant!(self, Self::Always)
    }

    /// Gets the access token for an endpoint that requires one for appservices.
    ///
    /// Returns `Some(_)` if `self` is either `SendAccessToken::Appservice(_)`
    /// or `SendAccessToken::Always(_)`
    pub fn get_required_for_appservice(self) -> Option<&'a str> {
        as_variant!(self, Self::Appservice | Self::Always)
    }
}

/// An error that can occur when adding an [`AuthScheme`] that requires an access token.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddRequiredTokenError {
    /// No access token was provided, but the endpoint requires one.
    #[error("no access token provided, but this endpoint requires one")]
    MissingAccessToken,

    /// Failed to convert the authentication to a header value.
    #[error(transparent)]
    IntoHeader(#[from] header::InvalidHeaderValue),
}

/// An error that can occur when extracting an [`AuthScheme`] that expects an access token.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExtractTokenError {
    /// No access token was found, but the endpoint requires one.
    #[error("no access token found, but this endpoint requires one")]
    MissingAccessToken,

    /// Failed to convert the header value to a UTF-8 string.
    #[error(transparent)]
    FromHeader(#[from] header::ToStrError),

    /// The scheme of the Authorization HTTP header is invalid.
    #[error("invalid authorization header scheme")]
    InvalidAuthorizationScheme,

    /// Failed to deserialize the query string.
    #[error("failed to deserialize query string: {0}")]
    FromQuery(#[from] serde_html_form::de::Error),
}
