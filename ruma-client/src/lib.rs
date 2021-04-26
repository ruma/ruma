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
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use assign::assign;
use http::uri::Uri;
use ruma_api::{AuthScheme, OutgoingRequest, SendAccessToken};
use ruma_serde::urlencoded;

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
    http_client::{DefaultConstructibleHttpClient, HttpClient},
};

/// A client for the Matrix client-server API.
#[derive(Clone, Debug)]
pub struct Client<C>(Arc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData<C> {
    /// The URL of the homeserver to connect to.
    homeserver_url: Uri,

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
        homeserver_url: Uri,
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
    pub fn new(homeserver_url: Uri, access_token: Option<String>) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            http_client: DefaultConstructibleHttpClient::default(),
            access_token: Mutex::new(access_token),
        }))
    }
}

impl<C: HttpClient> Client<C> {
    /// Makes a request to a Matrix API endpoint.
    pub async fn request<Request: OutgoingRequest>(
        &self,
        request: Request,
    ) -> Result<Request::IncomingResponse, Error<C::Error, Request::EndpointError>> {
        self.request_with_url_params(request, None).await
    }

    /// Makes a request to a Matrix API endpoint including additional URL parameters.
    pub async fn request_with_url_params<Request: OutgoingRequest>(
        &self,
        request: Request,
        extra_params: Option<BTreeMap<String, String>>,
    ) -> Result<Request::IncomingResponse, Error<C::Error, Request::EndpointError>> {
        let client = self.0.clone();
        let mut http_request = {
            let lock;
            let access_token = if Request::METADATA.authentication == AuthScheme::AccessToken {
                lock = client.access_token.lock().unwrap();
                if let Some(access_token) = &*lock {
                    SendAccessToken::IfRequired(access_token.as_str())
                } else {
                    return Err(Error::AuthenticationRequired);
                }
            } else {
                SendAccessToken::None
            };

            request.try_into_http_request(&client.homeserver_url.to_string(), access_token)?
        };

        let extra_params = urlencoded::to_string(extra_params).unwrap();
        let uri = http_request.uri_mut();
        let new_path_and_query = match uri.query() {
            Some(params) => format!("{}?{}&{}", uri.path(), params, extra_params),
            None => format!("{}?{}", uri.path(), extra_params),
        };
        *uri = Uri::from_parts(assign!(uri.clone().into_parts(), {
            path_and_query: Some(new_path_and_query.parse()?),
        }))?;

        let http_response =
            client.http_client.send_http_request(http_request).await.map_err(Error::Response)?;
        Ok(ruma_api::IncomingResponse::try_from_http_response(http_response)?)
    }
}
