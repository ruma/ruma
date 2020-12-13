//! Crate `ruma_api` contains core types used to define the requests and responses for each endpoint
//! in the various [Matrix](https://matrix.org) API specifications.
//! These types can be shared by client and server code for all Matrix APIs.
//!
//! When implementing a new Matrix API, each endpoint has a request type which implements
//! `Endpoint`, and a response type connected via an associated type.
//!
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.

#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![warn(rust_2018_idioms)]
#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]

use std::{
    convert::{TryFrom, TryInto},
    error::Error as StdError,
};

use http::Method;

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
///     // The error returned when a response fails, defaults to `Void`.
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
///   the path that are parameterized can indicate a varible by using a Rust identifier
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
///             pub file: Vec<u8>,
///         }
///
///         response: {
///             #[ruma_api(body)]
///             pub my_custom_type: MyCustomType,
///         }
///     }
/// }
/// ```
///
/// ## Fallible deserialization
///
/// All request and response types also derive [`Outgoing`][Outgoing]. As such, to allow
/// fallible deserialization, you can use the `#[wrap_incoming]` attribute. For details, see
/// the documentation for [the derive macro](derive.Outgoing.html).
// TODO: Explain the concept of fallible deserialization before jumping to
// `ruma_serde::Outgoing`
pub use ruma_api_macros::ruma_api;

pub mod error;
/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-api's public API.
#[doc(hidden)]
pub mod exports {
    pub use http;
    pub use percent_encoding;
    pub use ruma_serde;
    pub use serde;
    pub use serde_json;
}

use error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError};

/// Gives users the ability to define their own serializable / deserializable errors.
pub trait EndpointError: StdError + Sized + 'static {
    /// Tries to construct `Self` from an `http::Response`.
    ///
    /// This will always return `Err` variant when no `error` field is defined in
    /// the `ruma_api` macro.
    fn try_from_response(
        response: http::Response<Vec<u8>>,
    ) -> Result<Self, error::ResponseDeserializationError>;
}

/// A request type for a Matrix API endpoint. (trait used for sending requests)
pub trait OutgoingRequest {
    /// A type capturing the expected error conditions the server can return.
    type EndpointError: EndpointError;

    /// Response type returned when the request is successful.
    type IncomingResponse: TryFrom<
        http::Response<Vec<u8>>,
        Error = FromHttpResponseError<Self::EndpointError>,
    >;

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
    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, IntoHttpError>;
}

/// A request type for a Matrix API endpoint. (trait used for receiving requests)
pub trait IncomingRequest: Sized {
    /// A type capturing the error conditions that can be returned in the response.
    type EndpointError: EndpointError;

    /// Response type to return when the request is successful.
    type OutgoingResponse: TryInto<http::Response<Vec<u8>>, Error = IntoHttpError>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;

    /// Tries to turn the given `http::Request` into this request type.
    fn try_from_http_request(req: http::Request<Vec<u8>>) -> Result<Self, FromHttpRequestError>;
}

/// Marker trait for requests that don't require authentication. (for the client side)
pub trait OutgoingNonAuthRequest: OutgoingRequest {}

/// Marker trait for requests that don't require authentication. (for the server side)
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

#[doc(hidden)]
#[macro_export]
macro_rules! try_deserialize {
    ($kind:ident, $call:expr $(,)?) => {
        $crate::try_deserialize!(@$kind, $kind, $call)
    };
    (@request, $kind:ident, $call:expr) => {
        match $call {
            Ok(val) => val,
            Err(err) => return Err($crate::error::RequestDeserializationError::new(err, $kind).into()),
        }
    };
    (@response, $kind:ident, $call:expr) => {
        match $call {
            Ok(val) => val,
            Err(err) => return Err($crate::error::ResponseDeserializationError::new(err, $kind).into()),
        }
    };
}
