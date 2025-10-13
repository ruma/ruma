//! The `AuthScheme` trait used to specify the authentication scheme used by endpoints and the types
//! that implement it.

#![allow(clippy::exhaustive_structs)]

use http::{header, HeaderName, HeaderValue};

use crate::api::{error::IntoHttpError, SendAccessToken};

/// Trait implemented by types representing an authentication scheme used by an endpoint.
pub trait AuthScheme: Sized {
    /// The `Authorization` HTTP header to add to an outgoing request with this scheme.
    ///
    /// Transforms the `SendAccessToken` into an access token if the endpoint requires it, or if it
    /// is `SendAccessToken::Force`.
    ///
    /// Fails if the endpoint requires an access token but the parameter is `SendAccessToken::None`,
    /// or if the access token can't be converted to a [`HeaderValue`].
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError>;
}

/// No authentication is performed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAuthentication;

impl AuthScheme for NoAuthentication {
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        access_token
            .get_not_required_for_endpoint()
            .map(access_token_to_authorization_header)
            .transpose()
    }
}

/// Authentication is performed by including an access token in the `Authentication` http
/// header, or an `access_token` query parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessToken;

impl AuthScheme for AccessToken {
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        let token =
            access_token.get_required_for_endpoint().ok_or(IntoHttpError::NeedsAuthentication)?;
        access_token_to_authorization_header(token).map(Some)
    }
}

/// Authentication is optional, and it is performed by including an access token in the
/// `Authentication` http header, or an `access_token` query parameter.
///
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessTokenOptional;

impl AuthScheme for AccessTokenOptional {
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        access_token
            .get_required_for_endpoint()
            .map(access_token_to_authorization_header)
            .transpose()
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
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        let token =
            access_token.get_required_for_appservice().ok_or(IntoHttpError::NeedsAuthentication)?;
        access_token_to_authorization_header(token).map(Some)
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
    fn authorization_header(
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        access_token
            .get_required_for_appservice()
            .map(access_token_to_authorization_header)
            .transpose()
    }
}

/// Authentication is performed by including X-Matrix signatures in the request headers,
/// as defined in the federation API.
#[derive(Debug, Clone, Copy, Default)]
pub struct ServerSignatures;

impl AuthScheme for ServerSignatures {
    fn authorization_header(
        _access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        Ok(None)
    }
}

/// Convert the given access token to an `Authorization` HTTP header.
fn access_token_to_authorization_header(
    token: &str,
) -> Result<(HeaderName, HeaderValue), IntoHttpError> {
    Ok((header::AUTHORIZATION, format!("Bearer {token}").try_into()?))
}
