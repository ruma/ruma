use http::{header, HeaderMap, HeaderValue};
use ruma_identifiers::{ServerNameBox, ServerSigningKeyId};
use ruma_serde::urlencoded;
use serde::Deserialize;

use super::{IncomingRequest, OutgoingRequest};
use crate::error::FromHttpRequestError;

/// Marker trait for requests that don't require authentication, for the client side.
pub trait OutgoingNonAuthRequest: OutgoingRequest {}

/// Marker trait for requests that don't require authentication, for the server side.
pub trait IncomingNonAuthRequest: IncomingRequest {}

/// Fields for `X-Matrix` authentication scheme
#[derive(Clone, Debug)]
pub struct MatrixAuthHeader {
    /// Sending homeserver
    pub origin: ServerNameBox,

    /// Key ID to use for checking signature
    pub key: ServerSigningKeyId,

    /// Signature of request
    pub signature: String,
}

/// Authentication data for incoming requests.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Authentication {
    /// Authentication data was not checked since the incoming request does not require
    /// authentication.
    NotRrequired,

    /// No authentication data found.
    None,

    /// An access token found in the `Authentication` header or the `access_token` query parameter.
    ///
    /// For `AuthScheme::AccessToken` (as opposed to `AuthScheme::QueryOnlyAccessToken`), this will
    /// contain the value found in the header if both are specified.
    AccessToken(String),

    // TODO: Implement Server Signature checking
    /// Signature for incoming federation request.
    ServerSignatures(Vec<MatrixAuthHeader>),
}

/// Authentication scheme used by the endpoint.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum AuthScheme {
    /// No authentication is performed.
    None,

    /// Authentication is performed by including an access token in the `Authentication` http
    /// header, or an `access_token` query parameter.
    ///
    /// It is recommended to use the header over the query parameter.
    AccessToken,

    /// Authentication is performed by including X-Matrix signatures in the request headers,
    /// as defined in the federation API.
    ServerSignatures,

    /// Authentication is performed by setting the `access_token` query parameter.
    QueryOnlyAccessToken,
}

#[derive(Deserialize)]
struct TokenStruct<'a> {
    access_token: Option<&'a str>,
}

pub fn extract_access_token<'a>(
    query: Option<&'a str>,
    headers: &'a HeaderMap<HeaderValue>,
) -> Result<Option<&'a str>, FromHttpRequestError> {
    match headers.get(header::AUTHORIZATION) {
        Some(auth_hdr) => auth_hdr
            .to_str()?
            .strip_prefix("Bearer ")
            .ok_or(FromHttpRequestError::NonBearerAuthHeader)
            .map(Some),
        None => extract_access_token_from_query(query).map_err(Into::into),
    }
}

pub fn extract_access_token_from_query(
    query: Option<&str>,
) -> Result<Option<&str>, urlencoded::de::Error> {
    Ok(match query {
        Some(q) => urlencoded::from_str::<TokenStruct<'_>>(q)?.access_token,
        None => None,
    })
}
