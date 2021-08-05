#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Core types used to define the requests and responses for each endpoint in the various
//! [Matrix API specifications][apis].
//!
//! When implementing a new Matrix API, each endpoint has a request type which implements
//! `Endpoint`, and a response type connected via an associated type.
//!
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.
//!
//! [apis]: https://matrix.org/docs/spec/#matrix-apis

#![warn(missing_docs)]

#[cfg(not(all(feature = "client", feature = "server")))]
compile_error!("ruma_api's Cargo features only exist as a workaround are not meant to be disabled");

use std::{convert::TryInto as _, error::Error as StdError};

use bytes::BufMut;
use http::Method;
use ruma_identifiers::UserId;

/// Generates a `ruma_api::Endpoint` from a concise definition.
///
/// The macro expects the following structure as input:
///
/// ```text
/// ruma_api! {
///     metadata: {
///         description: &'static str,
///         method: http::Method,
///         name: &'static str,
///         path: &'static str,
///         rate_limited: bool,
///         authentication: ruma_api::AuthScheme,
///     }
///
///     request: {
///         // Struct fields for each piece of data required
///         // to make a request to this API endpoint.
///     }
///
///     response: {
///         // Struct fields for each piece of data expected
///         // in the response from this API endpoint.
///     }
///
///     // The error returned when a response fails, defaults to `MatrixError`.
///     error: path::to::Error
/// }
/// ```
///
/// This will generate a `ruma_api::Metadata` value to be used for the `ruma_api::Endpoint`'s
/// associated constant, single `Request` and `Response` structs, and the necessary trait
/// implementations to convert the request into a `http::Request` and to create a response from
/// a `http::Response` and vice versa.
///
/// The details of each of the three sections of the macros are documented below.
///
/// ## Metadata
///
/// * `description`: A short description of what the endpoint does.
/// * `method`: The HTTP method used for requests to the endpoint. It's not necessary to import
///   `http::Method`'s associated constants. Just write the value as if it was imported, e.g.
///   `GET`.
/// * `name`: A unique name for the endpoint. Generally this will be the same as the containing
///   module.
/// * `path`: The path component of the URL for the endpoint, e.g. "/foo/bar". Components of
///   the path that are parameterized can indicate a variable by using a Rust identifier
///   prefixed with a colon, e.g. `/foo/:some_parameter`. A corresponding query string
///   parameter will be expected in the request struct (see below for details).
/// * `rate_limited`: Whether or not the endpoint enforces rate limiting on requests.
/// * `authentication`: What authentication scheme the endpoint uses.
///
/// ## Request
///
/// The request block contains normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There are also a few special attributes available to control how the struct is converted
/// into a `http::Request`:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the request. The value must implement `AsRef<str>`. Generally this is a
///   `String`. The attribute value shown above as `HEADER_NAME` must be a header name constant
///   from `http::header`, e.g. `CONTENT_TYPE`.
/// * `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///   component of the request URL.
/// * `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///   string.
/// * `#[ruma_api(query_map)]`: Instead of individual query fields, one query_map field, of any
///   type that implements `IntoIterator<Item = (String, String)>` (e.g. `HashMap<String,
///   String>`, can be used for cases where an endpoint supports arbitrary query parameters.
///
/// Any field that does not include one of these attributes will be part of the request's JSON
/// body.
///
/// ## Response
///
/// Like the request block, the response block consists of normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There is also a special attribute available to control how the struct is created from a
/// `http::Request`:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the response. The value must implement `AsRef<str>`. Generally this is a
///   `String`. The attribute value shown above as `HEADER_NAME` must be a header name constant
///   from `http::header`, e.g. `CONTENT_TYPE`.
///
/// Any field that does not include the above attribute will be expected in the response's JSON
/// body.
///
/// ## Newtype bodies
///
/// Both the request and response block also support "newtype bodies" by using the
/// `#[ruma_api(body)]` attribute on a field. If present on a field, the entire request or
/// response body will be treated as the value of the field. This allows you to treat the
/// entire request or response body as a specific type, rather than a JSON object with named
/// fields. Only one field in each struct can be marked with this attribute. It is an error to
/// have a newtype body field and normal body fields within the same struct.
///
/// There is another kind of newtype body that is enabled with `#[ruma_api(raw_body)]`. It is
/// used for endpoints in which the request or response body can be arbitrary bytes instead of
/// a JSON objects. A field with `#[ruma_api(raw_body)]` needs to have the type `Vec<u8>`.
///
/// # Examples
///
/// ```
/// pub mod some_endpoint {
///     use ruma_api_macros::ruma_api;
///
///     ruma_api! {
///         metadata: {
///             description: "Does something.",
///             method: POST,
///             name: "some_endpoint",
///             path: "/_matrix/some/endpoint/:baz",
///             rate_limited: false,
///             authentication: None,
///         }
///
///         request: {
///             pub foo: String,
///
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             #[ruma_api(query)]
///             pub bar: String,
///
///             #[ruma_api(path)]
///             pub baz: String,
///         }
///
///         response: {
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             pub value: String,
///         }
///     }
/// }
///
/// pub mod newtype_body_endpoint {
///     use ruma_api_macros::ruma_api;
///     use serde::{Deserialize, Serialize};
///
///     #[derive(Clone, Debug, Deserialize, Serialize)]
///     pub struct MyCustomType {
///         pub foo: String,
///     }
///
///     ruma_api! {
///         metadata: {
///             description: "Does something.",
///             method: PUT,
///             name: "newtype_body_endpoint",
///             path: "/_matrix/some/newtype/body/endpoint",
///             rate_limited: false,
///             authentication: None,
///         }
///
///         request: {
///             #[ruma_api(raw_body)]
///             pub file: &'a [u8],
///         }
///
///         response: {
///             #[ruma_api(body)]
///             pub my_custom_type: MyCustomType,
///         }
///     }
/// }
/// ```
pub use ruma_api_macros::ruma_api;

pub mod error;
/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-api's public API.
#[doc(hidden)]
pub mod exports {
    pub use bytes;
    pub use http;
    pub use percent_encoding;
    pub use ruma_api_macros;
    pub use ruma_serde;
    pub use serde;
    pub use serde_json;
}

use error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError};

/// An enum to control whether an access token should be added to outgoing requests
#[derive(Clone, Copy, Debug)]
pub enum SendAccessToken<'a> {
    /// Add the given access token to the request only if the `METADATA` on the request requires it
    IfRequired(&'a str),

    /// Always add the access token
    Always(&'a str),

    /// Don't add an access token. This will lead to an error if the request endpoint requires
    /// authentication
    None,
}

impl<'a> SendAccessToken<'a> {
    /// Get the access token for an endpoint that requires one.
    ///
    /// Returns `Some(_)` if `self` contains an access token.
    pub fn get_required_for_endpoint(self) -> Option<&'a str> {
        match self {
            Self::IfRequired(tok) | Self::Always(tok) => Some(tok),
            Self::None => None,
        }
    }

    /// Get the access token for an endpoint that should not require one.
    ///
    /// Returns `Some(_)` only if `self` is `SendAccessToken::Always(_)`.
    pub fn get_not_required_for_endpoint(self) -> Option<&'a str> {
        match self {
            Self::Always(tok) => Some(tok),
            Self::IfRequired(_) | Self::None => None,
        }
    }
}

/// A request type for a Matrix API endpoint, used for sending requests.
pub trait OutgoingRequest: Sized {
    /// A type capturing the expected error conditions the server can return.
    type EndpointError: EndpointError;

    /// Response type returned when the request is successful.
    type IncomingResponse: IncomingResponse<EndpointError = Self::EndpointError>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;

    /// Tries to convert this request into an `http::Request`.
    ///
    /// This method should only fail when called on endpoints that require authentication. It may
    /// also fail with a serialization error in case of bugs in Ruma though.
    ///
    /// The endpoints path will be appended to the given `base_url`, for example
    /// `https://matrix.org`. Since all paths begin with a slash, it is not necessary for the
    /// `base_url` to have a trailing slash. If it has one however, it will be ignored.
    fn try_into_http_request<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
    ) -> Result<http::Request<T>, IntoHttpError>;
}

/// A response type for a Matrix API endpoint, used for receiving responses.
pub trait IncomingResponse: Sized {
    /// A type capturing the expected error conditions the server can return.
    type EndpointError: EndpointError;

    /// Tries to convert the given `http::Response` into this response type.
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, FromHttpResponseError<Self::EndpointError>>;
}

/// An extension to `OutgoingRequest` which provides Appservice specific methods.
pub trait OutgoingRequestAppserviceExt: OutgoingRequest {
    /// Tries to convert this request into an `http::Request` and appends a virtual `user_id` to
    /// [assert Appservice identity][id_assert].
    ///
    /// [id_assert]: https://matrix.org/docs/spec/application_service/r0.1.2#identity-assertion
    fn try_into_http_request_with_user_id<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
        user_id: UserId,
    ) -> Result<http::Request<T>, IntoHttpError> {
        let mut http_request = self.try_into_http_request(base_url, access_token)?;
        let user_id_query =
            ruma_serde::urlencoded::to_string(&[("user_id", &user_id.into_string())])?;

        let uri = http_request.uri().to_owned();
        let mut parts = uri.into_parts();

        let path_and_query_with_user_id = match &parts.path_and_query {
            Some(path_and_query) => match path_and_query.query() {
                Some(_) => format!("{}&{}", path_and_query, user_id_query),
                None => format!("{}?{}", path_and_query, user_id_query),
            },
            None => format!("/?{}", user_id_query),
        };

        parts.path_and_query =
            Some(path_and_query_with_user_id.try_into().map_err(http::Error::from)?);

        *http_request.uri_mut() = parts.try_into().map_err(http::Error::from)?;

        Ok(http_request)
    }
}

impl<T: OutgoingRequest> OutgoingRequestAppserviceExt for T {}

/// A request type for a Matrix API endpoint, used for receiving requests.
pub trait IncomingRequest: Sized {
    /// A type capturing the error conditions that can be returned in the response.
    type EndpointError: EndpointError;

    /// Response type to return when the request is successful.
    type OutgoingResponse: OutgoingResponse;

    /// Metadata about the endpoint.
    const METADATA: Metadata;

    /// Tries to turn the given `http::Request` into this request type.
    fn try_from_http_request<T: AsRef<[u8]>>(
        req: http::Request<T>,
    ) -> Result<Self, FromHttpRequestError>;
}

/// A request type for a Matrix API endpoint, used for sending responses.
pub trait OutgoingResponse {
    /// Tries to convert this response into an `http::Response`.
    ///
    /// This method should only fail when when invalid header values are specified. It may also
    /// fail with a serialization error in case of bugs in Ruma though.
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError>;
}

/// Gives users the ability to define their own serializable / deserializable errors.
pub trait EndpointError: OutgoingResponse + StdError + Sized + Send + 'static {
    /// Tries to construct `Self` from an `http::Response`.
    ///
    /// This will always return `Err` variant when no `error` field is defined in
    /// the `ruma_api` macro.
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, error::ResponseDeserializationError>;
}

/// Marker trait for requests that don't require authentication, for the client side.
pub trait OutgoingNonAuthRequest: OutgoingRequest {}

/// Marker trait for requests that don't require authentication, for the server side.
pub trait IncomingNonAuthRequest: IncomingRequest {}

/// Authentication scheme used by the endpoint.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// Metadata about an API endpoint.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Metadata {
    /// A human-readable description of the endpoint.
    pub description: &'static str,

    /// The HTTP method used by this endpoint.
    pub method: Method,

    /// A unique identifier for this endpoint.
    pub name: &'static str,

    /// The path of this endpoint's URL, with variable names where path parameters should be filled
    /// in during a request.
    pub path: &'static str,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// What authentication scheme the server uses for this endpoint.
    pub authentication: AuthScheme,
}
