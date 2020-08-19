//! Crate `ruma_client` is a [Matrix](https://matrix.org/) client library.
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
//!     let client = Client::https(homeserver_url, None);
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
//! rather than calling `log_in`. This can also be used to create a session for an application service
//! that does not need to log in, but uses the access_token directly:
//!
//! ```no_run
//! use ruma_client::{Client, Session};
//!
//! let work = async {
//!     let homeserver_url = "https://example.com".parse().unwrap();
//!     let session = Session{access_token: "as_access_token".to_string(), identification: None};
//!     let client = Client::https(homeserver_url, Some(session));
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
//! # use futures_util::stream::{StreamExt as _, TryStreamExt as _};
//! # use ruma_client::Client;
//! # use ruma::presence::PresenceState;
//! # let homeserver_url = "https://example.com".parse().unwrap();
//! # let client = Client::https(homeserver_url, None);
//! # let next_batch_token = String::new();
//! # async {
//! let mut sync_stream = Box::pin(client.sync(
//!     None,
//!     next_batch_token,
//!     PresenceState::Online,
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
//! # let client = Client::https(homeserver_url, None);
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
#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]

use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::Duration,
};

use assign::assign;
use futures_core::stream::{Stream, TryStream};
use futures_util::stream;
use http::{uri::Uri, Response as HttpResponse};
use hyper::{client::HttpConnector, Client as HyperClient};
#[cfg(feature = "hyper-tls")]
use hyper_tls::HttpsConnector;
use ruma_api::{AuthScheme, OutgoingRequest};
use ruma_client_api::r0::{
    sync::sync_events::{Filter as SyncFilter, Request as SyncRequest, Response as SyncResponse},
    uiaa::AuthData,
};
use ruma_identifiers::DeviceId;
use ruma_serde::urlencoded;
use std::collections::BTreeMap;

mod error;
mod session;

pub use self::{
    error::Error,
    session::{Identification, Session},
};

/// A client for the Matrix client-server API.
#[derive(Debug)]
pub struct Client<C>(Arc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData<C> {
    /// The URL of the homeserver to connect to.
    homeserver_url: Uri,
    /// The underlying HTTP client.
    hyper: HyperClient<C>,
    /// User session data.
    session: Mutex<Option<Session>>,
}

/// Non-secured variant of the client (using plain HTTP requests)
pub type HttpClient = Client<HttpConnector>;

impl HttpClient {
    /// Creates a new client for making HTTP requests to the given homeserver.
    pub fn new(homeserver_url: Uri, session: Option<Session>) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            hyper: HyperClient::builder().build_http(),
            session: Mutex::new(session),
        }))
    }
}

/// Secured variant of the client (using HTTPS requests)
#[cfg(feature = "tls")]
pub type HttpsClient = Client<HttpsConnector<HttpConnector>>;

#[cfg(feature = "tls")]
impl HttpsClient {
    /// Creates a new client for making HTTPS requests to the given homeserver.
    pub fn https(homeserver_url: Uri, session: Option<Session>) -> Self {
        let connector = HttpsConnector::new();

        Self(Arc::new(ClientData {
            homeserver_url,
            hyper: HyperClient::builder().build(connector),
            session: Mutex::new(session),
        }))
    }
}

impl<C> Client<C>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    /// Creates a new client using the given `hyper::Client`.
    ///
    /// This allows the user to configure the details of HTTP as desired.
    pub fn custom(
        hyper_client: HyperClient<C>,
        homeserver_url: Uri,
        session: Option<Session>,
    ) -> Self {
        Self(Arc::new(ClientData {
            homeserver_url,
            hyper: hyper_client,
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
        use ruma_client_api::r0::session::login::{LoginInfo, Request as LoginRequest, UserInfo};

        let response = self
            .request(assign!(
                LoginRequest::new(UserInfo::MatrixId(user), LoginInfo::Password { password }), {
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
            .request(assign!(register::Request::new(), {
                auth: Some(AuthData::DirectRequest {
                    kind: "m.login.dummy",
                    session: None,
                    auth_parameters: BTreeMap::new(),
                }),
                username,
                password: Some(password),
            }))
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
        filter: Option<SyncFilter<'a>>,
        since: String,
        set_presence: ruma_common::presence::PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<SyncResponse, Error<ruma_client_api::Error>>>
           + TryStream<Ok = SyncResponse, Error = Error<ruma_client_api::Error>>
           + 'a {
        let client = self.clone();
        stream::try_unfold(since, move |since| {
            let client = client.clone();

            async move {
                let response = client
                    .request(assign!(SyncRequest::new(), {
                        filter,
                        since: Some(&since),
                        set_presence,
                        timeout,
                    }))
                    .await?;

                let next_batch_clone = response.next_batch.clone();
                Ok(Some((response, next_batch_clone)))
            }
        })
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

        // FIXME: We read the response into a contiguous buffer here (not actually required for
        // deserialization) and then copy the whole thing to convert from Bytes to Vec<u8>.
        let full_body = hyper::body::to_bytes(body).await?;
        let full_response = HttpResponse::from_parts(head, full_body.as_ref().to_owned());

        Ok(Request::IncomingResponse::try_from(full_response)?)
    }
}

impl<C> Clone for Client<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
