#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! A minimal [Matrix](https://matrix.org/) client library.
//!
//! # Usage
//!
//! Begin by creating a `Client` type, usually using the `https` method for a client that supports
//! secure connections, and then logging in:
//!
//! ```ignore
//! use ruma_client::Client;
//!
//! let work = async {
//!     let homeserver_url = "https://example.com".parse().unwrap();
//!     let client = Client::new(homeserver_url, None);
//!
//!     let session = client
//!         .log_in("@alice:example.com", "secret", None, None)
//!         .await?;
//!
//!     // You're now logged in! Write the session to a file if you want to restore it later.
//!     // Then start using the API!
//! # Result::<(), ruma_client::Error<_>>::Ok(())
//! };
//! ```
//!
//! You can also pass an existing access token to the `Client` constructor to restore a previous
//! session rather than calling `log_in`. This can also be used to create a session for an
//! application service that does not need to log in, but uses the access_token directly:
//!
//! ```ignore
//! use ruma_client::Client;
//!
//! let work = async {
//!     let homeserver_url = "https://example.com".parse().unwrap();
//!     let client = Client::new(homeserver_url, Some("as_access_token".into()));
//!
//!     // make calls to the API
//! };
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
//! ```ignore
//! # use ruma_client::Client;
//! # let homeserver_url = "https://example.com".parse().unwrap();
//! # let client = Client::new(homeserver_url, None);
//! use std::convert::TryFrom;
//!
//! use ruma::{
//!     api::client::r0::alias::get_alias,
//!     room_alias_id, room_id,
//! };
//!
//! async {
//!     let response = client
//!         .send_request(get_alias::Request::new(&room_alias_id!("#example_room:example.com")))
//!         .await?;
//!
//!     assert_eq!(response.room_id, room_id!("!n8f893n9:example.com"));
//! #   Result::<(), ruma_client::Error<_, _>>::Ok(())
//! }
//! # ;
//! ```

#![warn(rust_2018_idioms)]
#![deny(missing_debug_implementations, missing_docs)]

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use ruma_api::{OutgoingRequest, SendAccessToken};

// "Undo" rename from `Cargo.toml` that only serves to make `hyper-rustls` available as a Cargo
// feature name.
#[cfg(feature = "hyper-rustls")]
extern crate hyper_rustls_crate as hyper_rustls;

#[cfg(feature = "client-api")]
mod client_api;
mod error;
pub mod http_client;

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

/// A client for the Matrix client-server API.
#[derive(Clone, Debug)]
pub struct Client<C>(Arc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData<C> {
    /// The URL of the homeserver to connect to.
    homeserver_url: String,

    /// The underlying HTTP client.
    http_client: C,

    /// User session data.
    access_token: Mutex<Option<String>>,
}

impl<C> Client<C> {
    /// Creates a new client using the given underlying HTTP client.
    ///
    /// This allows the user to configure the details of HTTP as desired.
    pub fn with_http_client(
        http_client: C,
        homeserver_url: String,
        access_token: Option<String>,
    ) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            http_client,
            access_token: Mutex::new(access_token),
        }))
    }

    /// Get a copy of the current `access_token`, if any.
    ///
    /// Useful for serializing and persisting the session to be restored later.
    pub fn access_token(&self) -> Option<String> {
        self.0.access_token.lock().expect("session mutex was poisoned").clone()
    }
}

impl<C: DefaultConstructibleHttpClient> Client<C> {
    /// Creates a new client based on a default-constructed hyper HTTP client.
    pub fn new(homeserver_url: String, access_token: Option<String>) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            http_client: DefaultConstructibleHttpClient::default(),
            access_token: Mutex::new(access_token),
        }))
    }
}

impl<C: HttpClient> Client<C> {
    /// Makes a request to a Matrix API endpoint.
    pub async fn send_request<R: OutgoingRequest>(&self, request: R) -> ResponseResult<C, R>
    where
        <R as OutgoingRequest>::EndpointError: Send,
    {
        self.send_customized_request(request, |_| Ok(())).await
    }

    /// Makes a request to a Matrix API endpoint including additional URL parameters.
    pub async fn send_customized_request<R, F>(
        &self,
        request: R,
        customize: F,
    ) -> ResponseResult<C, R>
    where
        R: OutgoingRequest,
        <R as OutgoingRequest>::EndpointError: Send,
        F: FnOnce(&mut http::Request<C::RequestBody>) -> Result<(), ResponseError<C, R>>,
    {
        let access_token = self.access_token();
        let send_access_token = match access_token.as_deref() {
            Some(at) => SendAccessToken::IfRequired(at),
            None => SendAccessToken::None,
        };

        send_customized_request(
            &self.0.http_client,
            &self.0.homeserver_url,
            send_access_token,
            request,
            customize,
        )
        .await
    }
}

fn send_customized_request<'a, C, R, F>(
    http_client: &'a C,
    homeserver_url: &str,
    send_access_token: SendAccessToken<'_>,
    request: R,
    customize: F,
) -> impl Future<Output = ResponseResult<C, R>> + Send + 'a
where
    C: HttpClient + ?Sized,
    R: OutgoingRequest,
    <R as OutgoingRequest>::EndpointError: Send,
    F: FnOnce(&mut http::Request<C::RequestBody>) -> Result<(), ResponseError<C, R>>,
{
    let http_req = request
        .try_into_http_request(homeserver_url, send_access_token)
        .map_err(ResponseError::<C, R>::from)
        .and_then(|mut req| {
            customize(&mut req)?;
            Ok(req)
        });

    async move {
        let http_res = http_client.send_http_request(http_req?).await.map_err(Error::Response)?;
        Ok(ruma_api::IncomingResponse::try_from_http_response(http_res)?)
    }
}
