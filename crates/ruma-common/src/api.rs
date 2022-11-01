//! Core types used to define the requests and responses for each endpoint in the various
//! [Matrix API specifications][apis].
//!
//! When implementing a new Matrix API, each endpoint has a request type which implements
//! [`IncomingRequest`] and [`OutgoingRequest`], and a response type connected via an associated
//! type.
//!
//! An implementation of [`IncomingRequest`] or [`OutgoingRequest`] contains all the information
//! about the HTTP method, the path and input parameters for requests, and the structure of a
//! successful response. Such types can then be used by client code to make requests, and by server
//! code to fulfill those requests.
//!
//! [apis]: https://spec.matrix.org/v1.4/#matrix-apis

use std::{convert::TryInto as _, error::Error as StdError};

use bytes::BufMut;

use crate::UserId;

/// Generates [`IncomingRequest`] and [`OutgoingRequest`] from a concise definition.
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
///         authentication: ruma_common::api::AuthScheme,
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
/// This will generate a [`Metadata`] value to be used for the associated constants of
/// [`IncomingRequest`] and [`OutgoingRequest`], single `Request` and `Response` structs, and
/// the necessary trait implementations to convert the request into a `http::Request` and to
/// create a response from a `http::Response` and vice versa.
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
/// The request block contains normal struct field definitions. Doc comments and attributes are
/// allowed as normal. There are also a few special attributes available to control how the
/// struct is converted into an `http::Request`:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the request. The value must implement `AsRef<str>`. Generally this is a
///   `String`. The attribute value shown above as `HEADER_NAME` must be a `const` expression
///   of the type `http::header::HeaderName`, like one of the constants from `http::header`,
///   e.g. `CONTENT_TYPE`.
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
///     use http::header::CONTENT_TYPE;
///     use ruma_common::api::ruma_api;
///
///     ruma_api! {
///         metadata: {
///             description: "Does something.",
///             method: POST,
///             name: "some_endpoint",
///             stable_path: "/_matrix/some/endpoint/:baz",
///             rate_limited: false,
///             authentication: None,
///             added: 1.1,
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
///     use ruma_common::api::ruma_api;
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
///             stable_path: "/_matrix/some/newtype/body/endpoint",
///             rate_limited: false,
///             authentication: None,
///             added: 1.1,
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
pub use ruma_macros::ruma_api;

/// Generates [`OutgoingRequest`] and [`IncomingRequest`] implementations.
///
/// The `OutgoingRequest` impl is on the `Request` type this attribute is used on. It is
/// feature-gated behind `cfg(feature = "client")`.
///
/// The `IncomingRequest` impl is on `IncomingRequest`, which is either a type alias to
/// `Request` or a fully-owned version of the same, depending of whether `Request` has any
/// lifetime parameters. It is feature-gated behind `cfg(feature = "server")`.
///
/// The generated code expects a `METADATA` constant of type [`Metadata`] to be in scope,
/// alongside a `Response` type that implements [`OutgoingResponse`] (for
/// `cfg(feature = "server")`) and / or [`IncomingResponse`] (for `cfg(feature = "client")`).
///
/// ## Attributes
///
/// To declare which part of the request a field belongs to:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the request. The value must implement `Display`. Generally this is a `String`.
///   The attribute value shown above as `HEADER_NAME` must be a `const` expression of the type
///   `http::header::HeaderName`, like one of the constants from `http::header`, e.g.
///   `CONTENT_TYPE`.
/// * `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///   component of the request URL. If there are multiple of these fields, the order in which
///   they are declared must match the order in which they occur in the request path.
/// * `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///   string.
/// * `#[ruma_api(query_map)]`: Instead of individual query fields, one query_map field, of any
///   type that implements `IntoIterator<Item = (String, String)>` (e.g. `HashMap<String,
///   String>`, can be used for cases where an endpoint supports arbitrary query parameters.
/// * No attribute: Fields without an attribute are part of the body. They can use `#[serde]`
///   attributes to customize (de)serialization.
/// * `#[ruma_api(body)]`: Use this if multiple endpoints should share a request body type, or
///   the request body is better expressed as an `enum` rather than a `struct`. The value of
///   the field will be used as the JSON body (rather than being a field in the request body
///   object).
/// * `#[ruma_api(raw_body)]`: Like `body` in that the field annotated with it represents the
///   entire request body, but this attribute is for endpoints where the body can be anything,
///   not just JSON. The field type must be `&[u8]`.
///
/// ## Examples
///
/// ```
/// pub mod do_a_thing {
///     use ruma_common::{api::request, RoomId};
///     # use ruma_common::{
///     #     api::{response, Metadata},
///     #     metadata,
///     # };
///
///     // const METADATA: Metadata = metadata! { ... };
///     # const METADATA: Metadata = metadata! {
///     #     description: "Does something.",
///     #     method: POST,
///     #     name: "some_endpoint",
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/:room_id",
///     #     },
///     # };
///
///     #[request]
///     pub struct Request<'a> {
///         #[ruma_api(path)]
///         pub room_id: &'a RoomId,
///
///         #[ruma_api(query)]
///         pub bar: &'a str,
///
///         #[serde(default)]
///         pub foo: String,
///     }
///
///     // #[response]
///     // pub struct Response { ... }
///     # #[response]
///     # pub struct Response {}
/// }
///
/// pub mod upload_file {
///     use http::header::CONTENT_TYPE;
///     use ruma_common::api::request;
///     # use ruma_common::{
///     #     api::{response, Metadata},
///     #     metadata,
///     # };
///
///     // const METADATA: Metadata = metadata! { ... };
///     # const METADATA: Metadata = metadata! {
///     #     description: "Does something.",
///     #     method: POST,
///     #     name: "some_endpoint",
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/:file_name",
///     #     },
///     # };
///
///     #[request]
///     pub struct Request<'a> {
///         #[ruma_api(path)]
///         pub file_name: &'a str,
///
///         #[ruma_api(header = CONTENT_TYPE)]
///         pub content_type: String,
///
///         #[ruma_api(raw_body)]
///         pub file: &'a [u8],
///     }
///
///     // #[response]
///     // pub struct Response { ... }
///     # #[response]
///     # pub struct Response {}
/// }
/// ```
pub use ruma_macros::request;

/// Generates [`OutgoingResponse`] and [`IncomingResponse`] implementations.
///
/// The `OutgoingRequest` impl is feature-gated behind `cfg(feature = "client")`.
/// The `IncomingRequest` impl is feature-gated behind `cfg(feature = "server")`.
///
/// The generated code expects a `METADATA` constant of type [`Metadata`] to be in scope.
///
/// ## Attributes
///
/// To declare which part of the request a field belongs to:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the response. The value must implement `Display`. Generally this is a
///   `String`. The attribute value shown above as `HEADER_NAME` must be a header name constant
///   from `http::header`, e.g. `CONTENT_TYPE`.
/// * No attribute: Fields without an attribute are part of the body. They can use `#[serde]`
///   attributes to customize (de)serialization.
/// * `#[ruma_api(body)]`: Use this if multiple endpoints should share a response body type, or
///   the response body is better expressed as an `enum` rather than a `struct`. The value of
///   the field will be used as the JSON body (rather than being a field in the response body
///   object).
/// * `#[ruma_api(raw_body)]`: Like `body` in that the field annotated with it represents the
///   entire response body, but this attribute is for endpoints where the body can be anything,
///   not just JSON. The field type must be `Vec<u8>`.
///
/// ## Examples
///
/// ```
/// pub mod do_a_thing {
///     use ruma_common::{api::response, OwnedRoomId};
///     # use ruma_common::{
///     #     api::{request, Metadata},
///     #     metadata,
///     # };
///
///     // const METADATA: Metadata = metadata! { ... };
///     # const METADATA: Metadata = metadata! {
///     #     description: "Does something.",
///     #     method: POST,
///     #     name: "some_endpoint",
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint",
///     #     },
///     # };
///
///     // #[request]
///     // pub struct Request { ... }
///     # #[request]
///     # pub struct Request { }
///
///     #[response]
///     pub struct Response {
///         #[serde(skip_serializing_if = "Option::is_none")]
///         pub foo: Option<String>,
///     }
/// }
///
/// pub mod download_file {
///     use http::header::CONTENT_TYPE;
///     use ruma_common::api::response;
///     # use ruma_common::{
///     #     api::{request, Metadata},
///     #     metadata,
///     # };
///
///     // const METADATA: Metadata = metadata! { ... };
///     # const METADATA: Metadata = metadata! {
///     #     description: "Does something.",
///     #     method: POST,
///     #     name: "some_endpoint",
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint",
///     #     },
///     # };
///
///     // #[request]
///     // pub struct Request { ... }
///     # #[request]
///     # pub struct Request { }
///
///     #[response]
///     pub struct Response {
///         #[ruma_api(header = CONTENT_TYPE)]
///         pub content_type: String,
///
///         #[ruma_api(raw_body)]
///         pub file: Vec<u8>,
///     }
/// }
/// ```
pub use ruma_macros::response;

pub mod error;
mod metadata;

pub use metadata::{MatrixVersion, Metadata, VersionHistory, VersioningDecision};

use error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError};

/// An enum to control whether an access token should be added to outgoing requests
#[derive(Clone, Copy, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SendAccessToken<'a> {
    /// Add the given access token to the request only if the `METADATA` on the request requires
    /// it.
    IfRequired(&'a str),

    /// Always add the access token.
    Always(&'a str),

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
pub trait OutgoingRequest: Sized + Clone {
    /// A type capturing the expected error conditions the server can return.
    type EndpointError: EndpointError;

    /// Response type returned when the request is successful.
    type IncomingResponse: IncomingResponse<EndpointError = Self::EndpointError>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;

    /// Tries to convert this request into an `http::Request`.
    ///
    /// On endpoints with authentication, when adequate information isn't provided through
    /// access_token, this could result in an error. It may also fail with a serialization error
    /// in case of bugs in Ruma though.
    ///
    /// It may also fail if, for every version in `considering_versions`;
    /// - The endpoint is too old, and has been removed in all versions.
    ///   ([`EndpointRemoved`](error::IntoHttpError::EndpointRemoved))
    /// - The endpoint is too new, and no unstable path is known for this endpoint.
    ///   ([`NoUnstablePath`](error::IntoHttpError::NoUnstablePath))
    ///
    /// Finally, this will emit a warning through `tracing` if it detects if any version in
    /// `considering_versions` has deprecated this endpoint.
    ///
    /// The endpoints path will be appended to the given `base_url`, for example
    /// `https://matrix.org`. Since all paths begin with a slash, it is not necessary for the
    /// `base_url` to have a trailing slash. If it has one however, it will be ignored.
    fn try_into_http_request<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
        considering_versions: &'_ [MatrixVersion],
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

/// An extension to [`OutgoingRequest`] which provides Appservice specific methods.
pub trait OutgoingRequestAppserviceExt: OutgoingRequest {
    /// Tries to convert this request into an `http::Request` and appends a virtual `user_id` to
    /// [assert Appservice identity][id_assert].
    ///
    /// [id_assert]: https://spec.matrix.org/v1.4/application-service-api/#identity-assertion
    fn try_into_http_request_with_user_id<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
        user_id: &UserId,
        considering_versions: &'_ [MatrixVersion],
    ) -> Result<http::Request<T>, IntoHttpError> {
        let mut http_request =
            self.try_into_http_request(base_url, access_token, considering_versions)?;
        let user_id_query = crate::serde::urlencoded::to_string([("user_id", user_id)])?;

        let uri = http_request.uri().to_owned();
        let mut parts = uri.into_parts();

        let path_and_query_with_user_id = match &parts.path_and_query {
            Some(path_and_query) => match path_and_query.query() {
                Some(_) => format!("{path_and_query}&{user_id_query}"),
                None => format!("{path_and_query}?{user_id_query}"),
            },
            None => format!("/?{user_id_query}"),
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

    /// Tries to turn the given `http::Request` into this request type,
    /// together with the corresponding path arguments.
    ///
    /// Note: The strings in path_args need to be percent-decoded.
    fn try_from_http_request<B, S>(
        req: http::Request<B>,
        path_args: &[S],
    ) -> Result<Self, FromHttpRequestError>
    where
        B: AsRef<[u8]>,
        S: AsRef<str>;
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
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self;
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
}

/// Convenient constructor for [`Metadata`] constants.
///
/// Usage:
///
/// ```
/// # use ruma_common::{metadata, api::Metadata};
/// const _: Metadata = metadata! {
///     description: "Endpoint description.",
///     method: GET, // one of the associated constants of http::Method
///     name: "enpdoint_name",
///     rate_limited: true,
///     authentication: AccessToken, // one of the variants of api::AuthScheme
///
///     // history of endpoint paths
///     // there must be at least one path but otherwise everything is optional
///     history: {
///         unstable => "/_matrix/foo/org.bar.msc9000/baz",
///         unstable => "/_matrix/foo/org.bar.msc9000/qux",
///         1.0 => "/_matrix/media/r0/qux",
///         1.1 => "/_matrix/media/v3/qux",
///         1.2 => deprecated,
///         1.3 => removed,
///     }
/// };
/// ```
#[macro_export]
macro_rules! metadata {
    ( $( $field:ident: $rhs:tt ),+ $(,)? ) => {
        $crate::api::Metadata {
            $( $field: $crate::metadata!(@field $field: $rhs) ),+
        }
    };

    ( @field method: $method:ident ) => { $crate::exports::http::Method::$method };

    ( @field authentication: $scheme:ident ) => { $crate::api::AuthScheme::$scheme };

    ( @field history: {
        $( unstable => $unstable_path:literal, )*
        $( $( $version:literal => $rhs:tt, )+ )?
    } ) => {
        $crate::metadata! {
            @history_impl
            [ $($unstable_path),* ]
            // Flip left and right to avoid macro parsing ambiguities
            $( $( $rhs = $version ),+ )?
        }
    };

    // Simple literal case: used for description, name, rate_limited
    // Also used by ruma_api! while it still exists, for the history field
    ( @field $_field:ident: $rhs:expr ) => { $rhs };

    ( @history_impl
        [ $($unstable_path:literal),* ]
        $(
            $( $stable_path:literal = $version:literal ),+
            $(,
                deprecated = $deprecated_version:literal
                $(, removed = $removed_version:literal )?
            )?
        )?
    ) => {
        $crate::api::VersionHistory::new(
            &[ $( $unstable_path ),* ],
            &[ $($(
                ($crate::api::MatrixVersion::from_lit(stringify!($version)), $stable_path)
            ),+)? ],
            $crate::metadata!(@optional_version $($( $deprecated_version )?)?),
            $crate::metadata!(@optional_version $($($( $removed_version )?)?)?),
        )
    };

    ( @optional_version ) => { None };
    ( @optional_version $version:literal ) => { Some($crate::api::MatrixVersion::from_lit(stringify!($version))) }
}
