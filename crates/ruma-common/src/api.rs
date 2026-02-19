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

use bytes::BufMut;
/// Generates [`OutgoingRequest`] and [`IncomingRequest`] implementations.
///
/// The `OutgoingRequest` impl is feature-gated behind `cfg(feature = "client")`.
/// The `IncomingRequest` impl is feature-gated behind `cfg(feature = "server")`.
///
/// The generated code expects the `Request` type to implement [`Metadata`], alongside a
/// `Response` type that implements [`OutgoingResponse`] (for `cfg(feature = "server")`) and /
/// or [`IncomingResponse`] (for `cfg(feature = "client")`).
///
/// The `Content-Type` header of the `OutgoingRequest` is unset for endpoints using the `GET`
/// method, and defaults to `application/json` for all other methods, except if the `raw_body`
/// attribute is set on a field, in which case it defaults to `application/octet-stream`.
///
/// By default, the type this macro is used on gets a `#[non_exhaustive]` attribute. This
/// behavior can be controlled by setting the `ruma_unstable_exhaustive_types` compile-time
/// `cfg` setting as `--cfg=ruma_unstable_exhaustive_types` using `RUSTFLAGS` or
/// `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`). When that setting is
/// activated, the attribute is not applied so the type is exhaustive.
///
/// ## Container Attributes
///
/// * `#[request(error = ERROR_TYPE)]`: Override the `EndpointError` associated type of the
///   `OutgoingRequest` and `IncomingRequest` implementations. The default error type is
///   [`MatrixError`](error::MatrixError).
///
/// ## Field Attributes
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
///     use ruma_common::{RoomId, api::request};
///     # use ruma_common::{api::{auth_scheme::NoAuthentication, response}, metadata};
///
///     // metadata! { ... };
///     # metadata! {
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: NoAuthentication,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/{room_id}",
///     #     },
///     # }
///
///     #[request]
///     pub struct Request {
///         #[ruma_api(path)]
///         pub room_id: RoomId,
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
///     # use ruma_common::{api::{auth_scheme::NoAuthentication, response}, metadata};
///
///     // metadata! { ... };
///     # metadata! {
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: NoAuthentication,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint/{file_name}",
///     #     },
///     # }
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
/// The `Content-Type` header of the `OutgoingResponse` defaults to `application/json`, except
/// if the `raw_body` attribute is set on a field, in which case it defaults to
/// `application/octet-stream`.
///
/// By default, the type this macro is used on gets a `#[non_exhaustive]` attribute. This
/// behavior can be controlled by setting the `ruma_unstable_exhaustive_types` compile-time
/// `cfg` setting as `--cfg=ruma_unstable_exhaustive_types` using `RUSTFLAGS` or
/// `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`). When that setting is
/// activated, the attribute is not applied so the type is exhaustive.
///
/// ## Container Attributes
///
/// * `#[response(error = ERROR_TYPE)]`: Override the `EndpointError` associated type of the
///   `IncomingResponse` implementation. The default error type is
///   [`MatrixError`](error::MatrixError).
/// * `#[response(status = HTTP_STATUS)]`: Override the status code of `OutgoingResponse`.
///   `HTTP_STATUS` must be a status code constant from [`http::StatusCode`], e.g.
///   `IM_A_TEAPOT`. The default status code is [`200 OK`](http::StatusCode::OK);
///
/// ## Field Attributes
///
/// To declare which part of the response a field belongs to:
///
/// * `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///   headers on the response. `HEADER_NAME` must implement
///   `TryInto<http::header::HeaderName>`, this is usually a constant from [`http::header`].
///   The value of the field must implement `ToString` and `FromStr`, this is usually a
///   `String`. During deserialization of the response, if the field is an `Option` and parsing
///   the header fails, the error will be ignored and the value will be `None`.
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
///     use ruma_common::{RoomId, api::response};
///     # use ruma_common::{api::{auth_scheme::NoAuthentication, request}, metadata};
///
///     // metadata! { ... };
///     # metadata! {
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: NoAuthentication,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint",
///     #     },
///     # }
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
///     # use ruma_common::{api::{auth_scheme::NoAuthentication, request}, metadata};
///
///     // metadata! { ... };
///     # metadata! {
///     #     method: POST,
///     #     rate_limited: false,
///     #     authentication: NoAuthentication,
///     #     history: {
///     #         unstable => "/_matrix/some/endpoint",
///     #     },
///     # }
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
use crate::{DeviceId, UserId};

pub mod auth_scheme;
pub mod error;
mod metadata;
pub mod path_builder;

pub use self::metadata::{FeatureFlag, MatrixVersion, Metadata, SupportedVersions};

/// A request type for a Matrix API endpoint, used for sending requests.
pub trait OutgoingRequest: Metadata + Clone {
    /// A type capturing the expected error conditions the server can return.
    type EndpointError: EndpointError;

    /// Response type returned when the request is successful.
    type IncomingResponse: IncomingResponse<EndpointError = Self::EndpointError>;

    /// Tries to convert this request into an `http::Request`.
    ///
    /// The endpoints path will be appended to the given `base_url`, for example
    /// `https://matrix.org`. Since all paths begin with a slash, it is not necessary for the
    /// `base_url` to have a trailing slash. If it has one however, it will be ignored.
    ///
    /// ## Errors
    ///
    /// This method can return an error in the following cases:
    ///
    /// * On endpoints that require authentication, when adequate information isn't provided through
    ///   `authentication_input`, i.e. when [`AuthScheme::add_authentication()`] returns an error.
    /// * On endpoints that have several versions for the path, when there are no supported versions
    ///   for the endpoint, i.e. when [`PathBuilder::make_endpoint_url()`] returns an error.
    /// * If the request serialization fails, which should only happen in case of bugs in Ruma.
    ///
    /// [`AuthScheme::add_authentication()`]: auth_scheme::AuthScheme::add_authentication
    /// [`PathBuilder::make_endpoint_url()`]: path_builder::PathBuilder::make_endpoint_url
    fn try_into_http_request<T: Default + BufMut + AsRef<[u8]>>(
        self,
        base_url: &str,
        authentication_input: <Self::Authentication as auth_scheme::AuthScheme>::Input<'_>,
        path_builder_input: <Self::PathBuilder as path_builder::PathBuilder>::Input<'_>,
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
///
/// This is only implemented for implementors of [`AuthScheme`](auth_scheme::AuthScheme) that use a
/// [`SendAccessToken`](auth_scheme::SendAccessToken), because application services should only use
/// these methods with the Client-Server API.
pub trait OutgoingRequestAppserviceExt: OutgoingRequest
where
    for<'a> Self::Authentication:
        auth_scheme::AuthScheme<Input<'a> = auth_scheme::SendAccessToken<'a>>,
{
    /// Tries to convert this request into an `http::Request` and adds the given
    /// [`AppserviceUserIdentity`] to it, if the identity is not empty.
    fn try_into_http_request_with_identity<T: Default + BufMut + AsRef<[u8]>>(
        self,
        base_url: &str,
        access_token: auth_scheme::SendAccessToken<'_>,
        identity: AppserviceUserIdentity<'_>,
        path_builder_input: <Self::PathBuilder as path_builder::PathBuilder>::Input<'_>,
    ) -> Result<http::Request<T>, IntoHttpError> {
        let mut http_request =
            self.try_into_http_request(base_url, access_token, path_builder_input)?;

        identity.maybe_add_to_uri(http_request.uri_mut())?;

        Ok(http_request)
    }
}

impl<T: OutgoingRequest> OutgoingRequestAppserviceExt for T where
    for<'a> Self::Authentication:
        auth_scheme::AuthScheme<Input<'a> = auth_scheme::SendAccessToken<'a>>
{
}

/// A request type for a Matrix API endpoint, used for receiving requests.
pub trait IncomingRequest: Metadata {
    /// A type capturing the error conditions that can be returned in the response.
    type EndpointError: EndpointError;

    /// Response type to return when the request is successful.
    type OutgoingResponse: OutgoingResponse;

    /// Check whether the given HTTP method from an incoming request is compatible with the expected
    /// [`METHOD`](Metadata::METHOD) of this endpoint.
    fn check_request_method(method: &http::Method) -> Result<(), FromHttpRequestError> {
        if !(method == Self::METHOD
            || (Self::METHOD == http::Method::GET && method == http::Method::HEAD))
        {
            return Err(FromHttpRequestError::MethodMismatch {
                expected: Self::METHOD,
                received: method.clone(),
            });
        }

        Ok(())
    }

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

/// Data to [assert the identity] of an appservice virtual user.
///
/// [assert the identity]: https://spec.matrix.org/latest/application-service-api/#identity-assertion
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[non_exhaustive]
pub struct AppserviceUserIdentity<'a> {
    /// The ID of the virtual user.
    ///
    /// If this is not set, the user implied by the `sender_localpart` property of the registration
    /// will be used by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<&'a UserId>,

    /// The ID of a specific device belonging to the virtual user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<&'a DeviceId>,
}

impl<'a> AppserviceUserIdentity<'a> {
    /// Construct a new `AppserviceUserIdentity` with the given user ID.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id: Some(user_id), device_id: None }
    }

    /// Whether this identity is empty.
    fn is_empty(&self) -> bool {
        self.user_id.is_none() && self.device_id.is_none()
    }

    /// Add this identity to the given URI, if the identity is not empty.
    pub fn maybe_add_to_uri(&self, uri: &mut http::Uri) -> Result<(), IntoHttpError> {
        if self.is_empty() {
            // There will be no change to the URI.
            return Ok(());
        }

        // Serialize the query arguments of the identity.
        let identity_query = serde_html_form::to_string(self)?;

        // Add the query arguments to the URI.
        let mut parts = uri.clone().into_parts();

        let path_and_query_with_user_id = match &parts.path_and_query {
            Some(path_and_query) => match path_and_query.query() {
                Some(_) => format!("{path_and_query}&{identity_query}"),
                None => format!("{path_and_query}?{identity_query}"),
            },
            None => format!("/?{identity_query}"),
        };

        parts.path_and_query =
            Some(path_and_query_with_user_id.try_into().map_err(http::Error::from)?);

        *uri = parts.try_into().map_err(http::Error::from)?;

        Ok(())
    }
}
