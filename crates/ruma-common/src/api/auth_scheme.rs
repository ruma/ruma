//! The `AuthScheme` trait used to specify the authentication scheme used by endpoints and the types
//! that implement it.

#![allow(clippy::exhaustive_structs)]

use as_variant::as_variant;
use http::{header, HeaderMap};

use crate::api::error::IntoHttpError;

/// Trait implemented by types representing an authentication scheme used by an endpoint.
pub trait AuthScheme: Sized {
    /// The input necessary to generate the authentication.
    type Input<'a>;

    /// Add the appropriate header to the given outgoing request headers map for this
    /// authentication, if any.
    ///
    /// Returns an error if the endpoint requires authentication but the input doesn't provide it,
    /// or if the input can't be converted to a [`HeaderValue`](http::HeaderValue).
    fn add_authentication(
        headers: &mut HeaderMap,
        input: Self::Input<'_>,
    ) -> Result<(), IntoHttpError>;
}

/// No authentication is performed.
///
/// This type accepts a [`SendAccessToken`] as input to be able to send it regardless of whether it
/// is required.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAuthentication;

impl AuthScheme for NoAuthentication {
    type Input<'a> = SendAccessToken<'a>;

    fn add_authentication(
        headers: &mut HeaderMap,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), IntoHttpError> {
        if let Some(access_token) = access_token.get_not_required_for_endpoint() {
            add_access_token_as_authorization_header(headers, access_token)?;
        }

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

    fn add_authentication(
        headers: &mut HeaderMap,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), IntoHttpError> {
        let token =
            access_token.get_required_for_endpoint().ok_or(IntoHttpError::NeedsAuthentication)?;
        add_access_token_as_authorization_header(headers, token)
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

    fn add_authentication(
        headers: &mut HeaderMap,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), IntoHttpError> {
        if let Some(access_token) = access_token.get_required_for_endpoint() {
            add_access_token_as_authorization_header(headers, access_token)?;
        }

        Ok(())
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

    fn add_authentication(
        headers: &mut HeaderMap,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), IntoHttpError> {
        let token =
            access_token.get_required_for_appservice().ok_or(IntoHttpError::NeedsAuthentication)?;
        add_access_token_as_authorization_header(headers, token)
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

    fn add_authentication(
        headers: &mut HeaderMap,
        access_token: SendAccessToken<'_>,
    ) -> Result<(), IntoHttpError> {
        if let Some(access_token) = access_token.get_required_for_appservice() {
            add_access_token_as_authorization_header(headers, access_token)?;
        }

        Ok(())
    }
}

/// Authentication is performed by adding an `X-Matrix` header including a signature in the request
/// headers, as defined in the federation API.
///
/// Currently the `add_authentication` implementation is a noop, and the header must be computed and
/// added manually.
#[derive(Debug, Clone, Copy, Default)]
pub struct ServerSignatures;

impl AuthScheme for ServerSignatures {
    type Input<'a> = ();

    fn add_authentication(_headers: &mut HeaderMap, _input: ()) -> Result<(), IntoHttpError> {
        Ok(())
    }
}

/// Add the given access token as an `Authorization` HTTP header to the given map.
fn add_access_token_as_authorization_header(
    headers: &mut HeaderMap,
    token: &str,
) -> Result<(), IntoHttpError> {
    headers.insert(header::AUTHORIZATION, format!("Bearer {token}").try_into()?);
    Ok(())
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
