//! Crate `ruma-api-macros` provides a procedural macro for easily generating
//! [ruma-api](https://github.com/ruma/ruma-api)-compatible endpoints.
//!
//! See the documentation for the `ruma_api!` macro for usage details.

#![deny(missing_debug_implementations)]
#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate quote;
extern crate ruma_api;
extern crate syn;

use proc_macro::TokenStream;

use quote::ToTokens;

use api::{Api, RawApi};

mod api;

/// Generates a `ruma_api::Endpoint` from a concise definition.
///
/// The macro expects the following structure as input:
///
/// ```text
/// ruma_api! {
///     metadata {
///         description: &'static str
///         method: http::Method,
///         name: &'static str,
///         path: &'static str,
///         rate_limited: bool,
///         requires_authentication: bool,
///     }
///
///     request {
///         // Struct fields for each piece of data required
///         // to make a request to this API endpoint.
///     }
///
///     response {
///         // Struct fields for each piece of data expected
///         // in the response from this API endpoint.
///     }
/// }
/// # }
/// ```
///
/// This will generate a `ruma_api::Metadata` value to be used for the `ruma_api::Endpoint`'s
/// associated constant, single `Request` and `Response` structs, and the necessary trait
/// implementations to convert the request into a `http::Request` and to create a response from a
/// `http::Response`.
///
/// The details of each of the three sections of the macros are documented below.
///
/// ## Metadata
///
/// *   `description`: A short description of what the endpoint does.
/// *   `method`: The HTTP method used for requests to the endpoint.
///     It's not necessary to import `http::Method`'s associated constants. Just write
///     the value as if it was imported, e.g. `GET`.
/// *   `name`: A unique name for the endpoint.
///     Generally this will be the same as the containing module.
/// *   `path`: The path component of the URL for the endpoint, e.g. "/foo/bar".
///     Components of the path that are parameterized can indicate a varible by using a Rust
///     identifier prefixed with a colon, e.g. `/foo/:some_parameter`.
///     A corresponding query string parameter will be expected in the request struct (see below
///     for details).
/// *   `rate_limited`: Whether or not the endpoint enforces rate limiting on requests.
/// *   `requires_authentication`: Whether or not the endpoint requires a valid access token.
///
/// ## Request
///
/// The request block contains normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There are also a few special attributes available to control how the struct is converted into a
/// `http::Request`:
///
/// *   `#[ruma_api(header = "HEADER_NAME")]`: Fields with this attribute will be treated as HTTP
///     headers on the request.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
/// *   `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///     component of the request URL.
/// *   `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///     string.
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
/// *   `#[ruma_api(header = "HEADER_NAME")]`: Fields with this attribute will be treated as HTTP
///     headers on the response.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
///
/// Any field that does not include the above attribute will be expected in the response's JSON
/// body.
///
/// ## Newtype bodies
///
/// Both the request and response block also support "newtype bodies" by using the
/// `#[ruma_api(body)]` attribute on a field. If present on a field, the entire request or response
/// body will be treated as the value of the field. This allows you to treat the entire request or
/// response body as a specific type, rather than a JSON object with named fields. Only one field in
/// each struct can be marked with this attribute. It is an error to have a newtype body field and
/// normal body fields within the same struct.
///
/// # Examples
///
/// ```rust,no_run
/// #![feature(proc_macro, try_from)]
///
/// extern crate futures;
/// extern crate http;
/// extern crate ruma_api;
/// extern crate ruma_api_macros;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
/// extern crate serde_json;
/// extern crate serde_urlencoded;
/// extern crate url;
///
/// # fn main() {
/// pub mod some_endpoint {
///     use ruma_api_macros::ruma_api;
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: GET,
///             name: "some_endpoint",
///             path: "/_matrix/some/endpoint/:baz",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             pub foo: String,
///
///             #[ruma_api(header = "CONTENT_TYPE")]
///             pub content_type: String,
///
///             #[ruma_api(query)]
///             pub bar: String,
///
///             #[ruma_api(path)]
///             pub baz: String,
///         }
///
///         response {
///             #[ruma_api(header = "CONTENT_TYPE")]
///             pub content_type: String,
///
///             pub value: String,
///         }
///     }
/// }
///
/// pub mod newtype_body_endpoint {
///     use ruma_api_macros::ruma_api;
///
///     #[derive(Debug, Deserialize)]
///     pub struct MyCustomType {
///         pub foo: String,
///     }
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: GET,
///             name: "newtype_body_endpoint",
///             path: "/_matrix/some/newtype/body/endpoint",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             #[ruma_api(body)]
///             pub file: Vec<u8>,
///         }
///
///         response {
///             #[ruma_api(body)]
///             pub my_custom_type: MyCustomType,
///         }
///     }
/// }
/// # }
/// ```
#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = syn::parse_macro_input!(input as RawApi);

    let api = Api::from(raw_api);

    api.into_token_stream().into()
}
