#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! A [Matrix](https://matrix.org/) client library.
//!
//! # Usage
//!
//! Begin by creating a `Client` type, usually using the `https` method for a client that supports
//! secure connections, and then logging in:
//!
//! ```no_run
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
//! You can also pass an existing session to the `Client` constructor to restore a previous session
//! rather than calling `log_in`. This can also be used to create a session for an application
//! service that does not need to log in, but uses the access_token directly:
//!
//! ```no_run
//! use ruma_client::{Client, Session};
//!
//! let work = async {
//!     let homeserver_url = "https://example.com".parse().unwrap();
//!     let session = Session{access_token: "as_access_token".to_string(), identification: None};
//!     let client = Client::new(homeserver_url, Some(session));
//!
//!     // make calls to the API
//! };
//! ```
//!
//! For the standard use case of synchronizing with the homeserver (i.e. getting all the latest
//! events), use the `Client::sync`:
//!
//! ```no_run
//! use std::time::Duration;
//!
//! # use ruma_client::Client;
//! # use ruma::presence::PresenceState;
//! # use tokio_stream::{StreamExt as _};
//! # let homeserver_url = "https://example.com".parse().unwrap();
//! # let client = Client::new(homeserver_url, None);
//! # let next_batch_token = String::new();
//! # async {
//! let mut sync_stream = Box::pin(client.sync(
//!     None,
//!     next_batch_token,
//!     &PresenceState::Online,
//!     Some(Duration::from_secs(30)),
//! ));
//! while let Some(response) = sync_stream.try_next().await? {
//!     // Do something with the data in the response...
//! }
//! # Result::<(), ruma_client::Error<_>>::Ok(())
//! # };
//! ```
//!
//! The `Client` type also provides methods for registering a new account if you don't already have
//! one with the given homeserver.
//!
//! Beyond these basic convenience methods, `ruma-client` gives you access to the entire Matrix
//! client-server API via the `api` module. Each leaf module under this tree of modules contains
//! the necessary types for one API endpoint. Simply call the module's `call` method, passing it
//! the logged in `Client` and the relevant `Request` type. `call` will return a future that will
//! resolve to the relevant `Response` type.
//!
//! For example:
//!
//! ```no_run
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
//!         .request(get_alias::Request::new(&room_alias_id!("#example_room:example.com")))
//!         .await?;
//!
//!     assert_eq!(response.room_id, room_id!("!n8f893n9:example.com"));
//! #   Result::<(), ruma_client::Error<_>>::Ok(())
//! }
//! # ;
//! ```

#![warn(rust_2018_idioms)]
#![deny(missing_debug_implementations, missing_docs)]

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use assign::assign;
use async_stream::try_stream;
use futures_core::stream::Stream;
use http::{uri::Uri, Response as HttpResponse};
use hyper::client::{Client as HyperClient, HttpConnector};
use ruma_api::{AuthScheme, OutgoingRequest};
use ruma_client_api::r0::sync::sync_events::{
    Filter as SyncFilter, Request as SyncRequest, Response as SyncResponse,
};
use ruma_common::presence::PresenceState;
use ruma_identifiers::DeviceId;
use ruma_serde::urlencoded;

mod error;
mod session;

pub use self::{
    error::Error,
    session::{Identification, Session},
};

#[cfg(not(feature = "_tls"))]
type Connector = HttpConnector;

#[cfg(feature = "tls-native")]
type Connector = hyper_tls::HttpsConnector<HttpConnector>;

#[cfg(feature = "_tls-rustls")]
type Connector = hyper_rustls::HttpsConnector<HttpConnector>;

fn create_connector() -> Connector {
    #[cfg(not(feature = "_tls"))]
    let connector = HttpConnector::new();

    #[cfg(feature = "tls-native")]
    let connector = hyper_tls::HttpsConnector::new();

    #[cfg(feature = "tls-rustls-native-roots")]
    let connector = hyper_rustls::HttpsConnector::with_native_roots();

    #[cfg(feature = "tls-rustls-webpki-roots")]
    let connector = hyper_rustls::HttpsConnector::with_webpki_roots();

    connector
}

/// A client for the Matrix client-server API.
#[derive(Clone, Debug)]
pub struct Client(Arc<ClientData>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData {
    /// The URL of the homeserver to connect to.
    homeserver_url: Uri,

    /// The underlying HTTP client.
    hyper: HyperClient<Connector>,

    /// User session data.
    session: Mutex<Option<Session>>,
}

impl Client {
    /// Creates a new client.
    pub fn new(homeserver_url: Uri, session: Option<Session>) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            hyper: HyperClient::builder().build(create_connector()),
            session: Mutex::new(session),
        }))
    }

    /// Creates a new client using the given `hyper::client::Builder`.
    ///
    /// This allows the user to configure the details of HTTP as desired.
    pub fn custom(
        client_builder: &hyper::client::Builder,
        homeserver_url: Uri,
        session: Option<Session>,
    ) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            hyper: client_builder.build(create_connector()),
            session: Mutex::new(session),
        }))
    }

    /// Get a copy of the current `Session`, if any.
    ///
    /// Useful for serializing and persisting the session to be restored later.
    pub fn session(&self) -> Option<Session> {
        self.0.session.lock().expect("session mutex was poisoned").clone()
    }

    /// Log in with a username and password.
    ///
    /// In contrast to `api::r0::session::login::call()`, this method stores the
    /// session data returned by the endpoint in this client, instead of
    /// returning it.
    pub async fn log_in(
        &self,
        user: &str,
        password: &str,
        device_id: Option<&DeviceId>,
        initial_device_display_name: Option<&str>,
    ) -> Result<Session, Error<ruma_client_api::Error>> {
        use ruma_client_api::r0::session::login::{
            LoginInfo, Request as LoginRequest, UserIdentifier,
        };

        let response = self
            .request(assign!(
                LoginRequest::new(
                    LoginInfo::Password { identifier: UserIdentifier::MatrixId(user), password }
                ), {
                    device_id,
                    initial_device_display_name,
                }
            ))
            .await?;

        let session = Session {
            access_token: response.access_token,
            identification: Some(Identification {
                device_id: response.device_id,
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Register as a guest. In contrast to `api::r0::account::register::call()`,
    /// this method stores the session data returned by the endpoint in this
    /// client, instead of returning it.
    pub async fn register_guest(
        &self,
    ) -> Result<Session, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        use ruma_client_api::r0::account::register::{self, RegistrationKind};

        let response = self
            .request(assign!(register::Request::new(), { kind: RegistrationKind::Guest }))
            .await?;

        let session = Session {
            // since we supply inhibit_login: false above, the access token needs to be there
            // TODO: maybe unwrap is not the best solution though
            access_token: response.access_token.unwrap(),
            identification: Some(Identification {
                // same as access_token
                device_id: response.device_id.unwrap(),
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Register as a new user on this server.
    ///
    /// In contrast to `api::r0::account::register::call()`, this method stores
    /// the session data returned by the endpoint in this client, instead of
    /// returning it.
    ///
    /// The username is the local part of the returned user_id. If it is
    /// omitted from this request, the server will generate one.
    pub async fn register_user(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<Session, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        use ruma_client_api::r0::account::register;

        let response = self
            .request(assign!(register::Request::new(), { username, password: Some(password) }))
            .await?;

        let session = Session {
            // since we supply inhibit_login: false above, the access token needs to be there
            // TODO: maybe unwrap is not the best solution though
            access_token: response.access_token.unwrap(),
            identification: Some(Identification {
                // same as access_token
                device_id: response.device_id.unwrap(),
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    pub fn sync<'a>(
        &self,
        filter: Option<&'a SyncFilter<'a>>,
        mut since: String,
        set_presence: &'a PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<SyncResponse, Error<ruma_client_api::Error>>> + 'a {
        let client = self.clone();
        try_stream! {
            loop {
                let response = client
                    .request(assign!(SyncRequest::new(), {
                        filter,
                        since: Some(&since),
                        set_presence,
                        timeout,
                    }))
                    .await?;

                since = response.next_batch.clone();
                yield response;
            }
        }
    }

    /// Makes a request to a Matrix API endpoint.
    pub async fn request<Request: OutgoingRequest>(
        &self,
        request: Request,
    ) -> Result<Request::IncomingResponse, Error<Request::EndpointError>> {
        self.request_with_url_params(request, None).await
    }

    /// Makes a request to a Matrix API endpoint including additional URL parameters.
    pub async fn request_with_url_params<Request: OutgoingRequest>(
        &self,
        request: Request,
        extra_params: Option<BTreeMap<String, String>>,
    ) -> Result<Request::IncomingResponse, Error<Request::EndpointError>> {
        let client = self.0.clone();
        let mut http_request = {
            let session;
            let access_token = if Request::METADATA.authentication == AuthScheme::AccessToken {
                session = client.session.lock().unwrap();
                if let Some(s) = &*session {
                    Some(s.access_token.as_str())
                } else {
                    return Err(Error::AuthenticationRequired);
                }
            } else {
                None
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

        let hyper_response = client.hyper.request(http_request.map(hyper::Body::from)).await?;
        let (head, body) = hyper_response.into_parts();

        let full_body = hyper::body::aggregate(body).await?;
        let full_response = HttpResponse::from_parts(head, full_body);

        Ok(ruma_api::IncomingResponse::try_from_http_response(full_response)?)
    }
}
