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
//! [apis]: https://spec.matrix.org/latest/#matrix-apis

use std::{convert::TryInto as _, error::Error as StdError};

use as_variant::as_variant;
use bytes::BufMut;
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
/// By default, the type this macro is used on gets a `#[non_exhaustive]` attribute. This
/// behavior can be controlled by setting the `ruma_unstable_exhaustive_types` compile-time
/// `cfg` setting as `--cfg=ruma_unstable_exhaustive_types` using `RUSTFLAGS` or
/// `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`). When that setting is
/// activated, the attribute is not applied so the type is exhaustive.
///
/// ## Attributes
///
/// To declare which part of the request a field belongs to:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the request. The value must implement `ToString` and `FromStr`. Generally this
///   is a `String`. The attribute value shown above as `HEADER_NAME` must be a `const`
///   expression of the type `http::header::HeaderName`, like one of the constants from
///   `http::header`, e.g. `CONTENT_TYPE`. During deserialization of the request, if the field
///   is an `Option` and parsing the header fails, the error will be ignored and the value will
///   be `None`.
/// * `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///   component of the request URL. If there are multiple of these fields, the order in which
///   they are declared must match the order in which they occur in the request path.
/// * `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///   string.
/// * `#[ruma_api(query_all)]`: Instead of individual query fields, one query_all field, of any
///   type that can be (de)serialized by [serde_html_form], can be used for cases where
///   multiple endpoints should share a query fields type, the query fields are better
///   expressed as an `enum` rather than a `struct`, or the endpoint supports arbitrary query
///   parameters.
/// * No attribute: Fields without an attribute are part of the body. They can use `#[serde]`
///   attributes to customize (de)serialization.
/// * `#[ruma_api(body)]`: Use this if multiple endpoints should share a request body type, or
///   the request body is better expressed as an `enum` rather than a `struct`. The value of
///   the field will be used as the JSON body (rather than being a field in the request body
///   object).
/// * `#[ruma_api(raw_body)]`: Like `body` in that the field annotated with it represents the
///   entire request body, but this attribute is for endpoints where the body can be anything,
///   not just JSON. The field type must be `Vec<u8>`.
///
/// ## Examples
///
/// ```
/// pub mod do_a_thing {
///     use ruma_common::{api::request, OwnedRoomId};
///     # use ruma_common::{
///     #     api::{response, Metadata},
///     #     metadata,
///     # };
///
///     // const METADATA: Metadata = metadata! { ... };
///     # const METADATA: Metadata = metadata! {
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/{room_id}",
///     #     },
///     # };
///
///     #[request]
///     pub struct Request {
///         #[ruma_api(path)]
///         pub room_id: OwnedRoomId,
///
///         #[ruma_api(query)]
///         pub bar: String,
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
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: None,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/{file_name}",
///     #     },
///     # };
///
///     #[request]
///     pub struct Request {
///         #[ruma_api(path)]
///         pub file_name: String,
///
///         #[ruma_api(header = CONTENT_TYPE)]
///         pub content_type: String,
///
///         #[ruma_api(raw_body)]
///         pub file: Vec<u8>,
///     }
///
///     // #[response]
///     // pub struct Response { ... }
///     # #[response]
///     # pub struct Response {}
/// }
/// ```
///
/// [serde_html_form]: https://crates.io/crates/serde_html_form
pub use ruma_macros::request;
/// Generates [`OutgoingResponse`] and [`IncomingResponse`] implementations.
///
/// The `OutgoingResponse` impl is feature-gated behind `cfg(feature = "server")`.
/// The `IncomingResponse` impl is feature-gated behind `cfg(feature = "client")`.
///
/// The generated code expects a `METADATA` constant of type [`Metadata`] to be in scope.
///
/// By default, the type this macro is used on gets a `#[non_exhaustive]` attribute. This
/// behavior can be controlled by setting the `ruma_unstable_exhaustive_types` compile-time
/// `cfg` setting as `--cfg=ruma_unstable_exhaustive_types` using `RUSTFLAGS` or
/// `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`). When that setting is
/// activated, the attribute is not applied so the type is exhaustive.
///
/// The status code of `OutgoingResponse` can be optionally overridden by adding the `status`
/// attribute to `response`. The attribute value must be a status code constant from
/// `http::StatusCode`, e.g. `IM_A_TEAPOT`.
///
/// ## Attributes
///
/// To declare which part of the response a field belongs to:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the response. The value must implement `ToString` and `FromStr`. Generally
///   this is a `String`. The attribute value shown above as `HEADER_NAME` must be a header
///   name constant from `http::header`, e.g. `CONTENT_TYPE`. During deserialization of the
///   response, if the field is an `Option` and parsing the header fails, the error will be
///   ignored and the value will be `None`.
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
///     #     method: POST,
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
///     #[response(status = IM_A_TEAPOT)]
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
///     #     method: POST,
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
use serde::{Deserialize, Serialize};

use self::error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError};
#[doc(inline)]
pub use crate::metadata;
use crate::UserId;

pub mod error;
mod metadata;

pub use self::metadata::{
    FeatureFlag, MatrixVersion, Metadata, StablePathSelector, SupportedVersions, VersionHistory,
    VersioningDecision,
};

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
    /// It may also fail if, for every version in `considering`;
    /// - The endpoint is too old, and has been removed in all versions.
    ///   ([`EndpointRemoved`](error::IntoHttpError::EndpointRemoved))
    /// - The endpoint is too new, and no unstable path is known for this endpoint.
    ///   ([`NoUnstablePath`](error::IntoHttpError::NoUnstablePath))
    ///
    /// Finally, this will emit a warning through [`tracing`] if it detects that any version in
    /// `considering` has deprecated this endpoint.
    ///
    /// The endpoints path will be appended to the given `base_url`, for example
    /// `https://matrix.org`. Since all paths begin with a slash, it is not necessary for the
    /// `base_url` to have a trailing slash. If it has one however, it will be ignored.
    fn try_into_http_request<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
        considering: &'_ SupportedVersions,
    ) -> Result<http::Request<T>, IntoHttpError>;

    /// Whether the homeserver advertises support for this endpoint.
    ///
    /// Returns `true` if any version or feature in the given [`SupportedVersions`] matches a path
    /// in the history of this endpoint, unless the endpoint was removed.
    ///
    /// Note that this is likely to return false negatives, since some endpoints don't specify a
    /// stable or unstable feature, and homeservers should not advertise support for a Matrix
    /// version unless they support all of its features.
    fn is_supported(considering_versions: &SupportedVersions) -> bool {
        Self::METADATA.history.is_supported(considering_versions)
    }
}

/// A response type for a Matrix API endpoint, used for receiving responses.
pub trait IncomingResponse: Sized + Send {
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
    /// [id_assert]: https://spec.matrix.org/latest/application-service-api/#identity-assertion
    fn try_into_http_request_with_user_id<T: Default + BufMut>(
        self,
        base_url: &str,
        access_token: SendAccessToken<'_>,
        user_id: &UserId,
        considering: &'_ SupportedVersions,
    ) -> Result<http::Request<T>, IntoHttpError> {
        let mut http_request = self.try_into_http_request(base_url, access_token, considering)?;
        let user_id_query = serde_html_form::to_string([("user_id", user_id)])?;

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
    /// Using the query parameter is deprecated since Matrix 1.11.
    AccessToken,

    /// Authentication is optional, and it is performed by including an access token in the
    /// `Authentication` http header, or an `access_token` query parameter.
    ///
    /// Using the query parameter is deprecated since Matrix 1.11.
    AccessTokenOptional,

    /// Authentication is required, and can only be performed for appservices, by including an
    /// appservice access token in the `Authentication` http header, or `access_token` query
    /// parameter.
    ///
    /// Using the query parameter is deprecated since Matrix 1.11.
    AppserviceToken,

    /// No authentication is performed for clients, but it can be performed for appservices, by
    /// including an appservice access token in the `Authentication` http header, or an
    /// `access_token` query parameter.
    ///
    /// Using the query parameter is deprecated since Matrix 1.11.
    AppserviceTokenOptional,

    /// Authentication is performed by including X-Matrix signatures in the request headers,
    /// as defined in the federation API.
    ServerSignatures,
}

/// The direction to return events from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[allow(clippy::exhaustive_enums)]
pub enum Direction {
    /// Return events backwards in time from the requested `from` token.
    #[default]
    #[serde(rename = "b")]
    Backward,

    /// Return events forwards in time from the requested `from` token.
    #[serde(rename = "f")]
    Forward,
}
