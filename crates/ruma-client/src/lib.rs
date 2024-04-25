#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! A minimal [Matrix](https://matrix.org/) client library.
//!
//! # Usage
//!
//! Begin by creating a `Client`, selecting one of the type aliases from `ruma_client::http_client`
//! for the generic parameter. For the client API, there are login and registration methods
//! provided for the client (feature `client-api`):
//!
//! ```ignore
//! # // HACK: "ignore" the doctest here because client.log_in needs client-api feature.
//! // type HttpClient = ruma_client::http_client::_;
//! # type HttpClient = ruma_client::http_client::Dummy;
//! # let work = async {
//! let homeserver_url = "https://example.com".to_owned();
//! let client = ruma::Client::builder()
//!     .homeserver_url(homeserver_url)
//!     .build::<ruma_client::http_client::Dummy>()
//!     .await?;
//!
//! let session = client
//!     .log_in("@alice:example.com", "secret", None, None)
//!     .await?;
//!
//! // You're now logged in! Write the session to a file if you want to restore it later.
//! // Then start using the API!
//! # Result::<(), ruma_client::Error<_, _>>::Ok(())
//! # };
//! ```
//!
//! You can also pass an existing access token to the `Client` constructor to restore a previous
//! session rather than calling `log_in`. This can also be used to create a session for an
//! application service that does not need to log in, but uses the access_token directly:
//!
//! ```no_run
//! # type HttpClient = ruma_client::http_client::Dummy;
//! #
//! # async {
//! let homeserver_url = "https://example.com".to_owned();
//! let client = ruma_client::Client::builder()
//!     .homeserver_url(homeserver_url)
//!     .access_token(Some("as_access_token".into()))
//!     .build::<HttpClient>()
//!     .await?;
//!
//! // make calls to the API
//! # Result::<(), ruma_client::Error<_, _>>::Ok(())
//! # };
//! ```
//!
//! The `Client` type also provides methods for registering a new account if you don't already have
//! one with the given homeserver.
//!
//! Beyond these basic convenience methods, `ruma-client` gives you access to the entire Matrix
//! client-server API via the `request` method. You can pass it any of the `Request` types found in
//! `ruma::api::*` and get back a corresponding response from the homeserver.
//!
//! For example:
//!
//! ```no_run
//! # let homeserver_url = "https://example.com".to_owned();
//! # async {
//! # let client = ruma_client::Client::builder()
//! #     .homeserver_url(homeserver_url)
//! #     .build::<ruma_client::http_client::Dummy>()
//! #     .await?;
//!
//! use ruma_client_api::alias::get_alias;
//! use ruma_common::{api::MatrixVersion, owned_room_alias_id, room_id};
//!
//! let alias = owned_room_alias_id!("#example_room:example.com");
//! let response = client.send_request(get_alias::v3::Request::new(alias)).await?;
//!
//! assert_eq!(response.room_id, room_id!("!n8f893n9:example.com"));
//! # Result::<(), ruma_client::Error<_, _>>::Ok(())
//! # };
//! ```
//!
//! # Crate features
//!
//! The following features activate http client types in the [`http_client`] module:
//!
//! * `hyper`
//! * `hyper-native-tls`
//! * `hyper-rustls`
//! * `reqwest` – if you use the `reqwest` library already, activate this feature and configure the
//!   TLS backend on `reqwest` directly. If you want to use `reqwest` but don't depend on it
//!   already, use one of the sub-features instead. For details on the meaning of these, see
//!   [reqwest's documentation](https://docs.rs/reqwest/0.11/reqwest/#optional-features):
//!   * `reqwest-native-tls`
//!   * `reqwest-native-tls-alpn`
//!   * `reqwest-native-tls-vendored`
//!   * `reqwest-rustls-manual-roots`
//!   * `reqwest-rustls-webpki-roots`
//!   * `reqwest-rustls-native-roots`

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::{any::type_name, future::Future};

use ruma_common::{
    api::{MatrixVersion, OutgoingRequest, SendAccessToken},
    UserId,
};
use tracing::{info_span, Instrument};

#[cfg(feature = "client-api")]
mod client;
mod error;
pub mod http_client;

#[cfg(feature = "client-api")]
pub use self::client::{Client, ClientBuilder};
pub use self::{
    error::Error,
    http_client::{DefaultConstructibleHttpClient, HttpClient, HttpClientExt},
};

/// The error type for sending the request `R` with the http client `C`.
pub type ResponseError<C, R> =
    Error<<C as HttpClient>::Error, <R as OutgoingRequest>::EndpointError>;

/// The result of sending the request `R` with the http client `C`.
pub type ResponseResult<C, R> =
    Result<<R as OutgoingRequest>::IncomingResponse, ResponseError<C, R>>;

fn send_customized_request<'a, C, R, F>(
    http_client: &'a C,
    homeserver_url: &str,
    send_access_token: SendAccessToken<'_>,
    for_versions: &[MatrixVersion],
    request: R,
    customize: F,
) -> impl Future<Output = ResponseResult<C, R>> + Send + 'a
where
    C: HttpClient + ?Sized,
    R: OutgoingRequest,
    F: FnOnce(&mut http::Request<C::RequestBody>) -> Result<(), ResponseError<C, R>>,
{
    let http_req =
        info_span!("serialize_request", request_type = type_name::<R>()).in_scope(move || {
            request
                .try_into_http_request(homeserver_url, send_access_token, for_versions)
                .map_err(ResponseError::<C, R>::from)
                .and_then(|mut req| {
                    customize(&mut req)?;
                    Ok(req)
                })
        });

    let send_span = info_span!(
        "send_request",
        request_type = type_name::<R>(),
        http_client = type_name::<C>(),
        homeserver_url,
    );

    async move {
        let http_res = http_client
            .send_http_request(http_req?)
            .instrument(send_span)
            .await
            .map_err(Error::Response)?;

        let res =
            info_span!("deserialize_response", response_type = type_name::<R::IncomingResponse>())
                .in_scope(move || {
                    ruma_common::api::IncomingResponse::try_from_http_response(http_res)
                })?;

        Ok(res)
    }
}

fn add_user_id_to_query<C: HttpClient + ?Sized, R: OutgoingRequest>(
    user_id: &UserId,
) -> impl FnOnce(&mut http::Request<C::RequestBody>) -> Result<(), ResponseError<C, R>> + '_ {
    use assign::assign;
    use http::uri::Uri;

    move |http_request| {
        let extra_params = serde_html_form::to_string([("user_id", user_id)]).unwrap();
        let uri = http_request.uri_mut();
        let new_path_and_query = match uri.query() {
            Some(params) => format!("{}?{params}&{extra_params}", uri.path()),
            None => format!("{}?{extra_params}", uri.path()),
        };
        *uri = Uri::from_parts(assign!(uri.clone().into_parts(), {
            path_and_query: Some(new_path_and_query.parse()?),
        }))?;

        Ok(())
    }
}
