#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Identity Service API][identity-api].
//! These types can be shared by client and identity service code.
//!
//! [identity-api]: https://spec.matrix.org/v1.18/identity-service-api/

#![warn(missing_docs)]

use ruma_common::api::auth_scheme::{AuthScheme, ExtractTokenError, add_access_token_as_authorization_header, extract_bearer_or_query_token};

pub mod association;
pub mod authentication;
pub mod discovery;
pub mod invitation;
pub mod keys;
pub mod lookup;
pub mod tos;

ruma_common::priv_owned_str!();

/// Authentication is required by including an identity server access token in the
/// `Authentication` http header, or an `access_token` query parameter.
/// 
/// Using the query parameter is deprecated since Matrix 1.11.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityServiceToken;

impl AuthScheme for IdentityServiceToken {
    type Input<'a> = &'a str;
    type AddAuthenticationError = http::header::InvalidHeaderValue;
    /// The identity service token.
    type Output = String;
    type ExtractAuthenticationError = ExtractTokenError;

    fn add_authentication<T: AsRef<[u8]>>(
        request: &mut http::Request<T>,
        access_token: Self::Input<'_>,
    ) -> Result<(), Self::AddAuthenticationError>
    {
        add_access_token_as_authorization_header(request.headers_mut(), access_token)
    }

    fn extract_authentication<T: AsRef<[u8]>>(
        request: &http::Request<T>,
    ) -> Result<Self::Output, Self::ExtractAuthenticationError>
    {
        extract_bearer_or_query_token(request)?.ok_or(ExtractTokenError::MissingAccessToken)
    }
}