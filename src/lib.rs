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
//!         .log_in("@alice:example.com".to_string(), "secret".to_string(), None, None)
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
//! # use ruma_common::presence::PresenceState;
//! # let homeserver_url = "https://example.com".parse().unwrap();
//! # let client = Client::https(homeserver_url, None);
//! # let next_batch_token = String::new();
//! # async {
//! let mut sync_stream = Box::pin(client.sync(
//!     None,
//!     Some(next_batch_token),
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
//! use ruma_client::api::r0::alias::get_alias;
//! use ruma_identifiers::{RoomAliasId, RoomId};
//!
//! async {
//!     let response = client
//!         .request(get_alias::Request {
//!             room_alias: RoomAliasId::try_from("#example_room:example.com").unwrap(),
//!         })
//!         .await?;
//!
//!     assert_eq!(response.room_id, RoomId::try_from("!n8f893n9:example.com").unwrap());
//! #   Result::<(), ruma_client::Error<_>>::Ok(())
//! }
//! # ;
//! ```

#![warn(rust_2018_idioms)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]

use std::{
    convert::TryFrom,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_core::{
    future::Future,
    stream::{Stream, TryStream},
};
use futures_util::stream;
use http::Response as HttpResponse;
use hyper::{client::HttpConnector, Client as HyperClient, Uri};
#[cfg(feature = "hyper-tls")]
use hyper_tls::HttpsConnector;
use ruma_api::Endpoint;
use ruma_identifiers::DeviceId;
use std::collections::BTreeMap;
use url::Url;

pub use ruma_client_api as api;
pub use ruma_events as events;
pub use ruma_identifiers as identifiers;

mod error;
mod session;

pub use self::{error::Error, session::Identification, session::Session};

/// A client for the Matrix client-server API.
#[derive(Debug)]
pub struct Client<C>(Arc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData<C> {
    /// The URL of the homeserver to connect to.
    homeserver_url: Url,
    /// The underlying HTTP client.
    hyper: HyperClient<C>,
    /// User session data.
    session: Mutex<Option<Session>>,
}

/// Non-secured variant of the client (using plain HTTP requests)
pub type HttpClient = Client<HttpConnector>;

impl HttpClient {
    /// Creates a new client for making HTTP requests to the given homeserver.
    pub fn new(homeserver_url: Url, session: Option<Session>) -> Self {
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
    pub fn https(homeserver_url: Url, session: Option<Session>) -> Self {
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
        homeserver_url: Url,
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
        self.0
            .session
            .lock()
            .expect("session mutex was poisoned")
            .clone()
    }

    /// Log in with a username and password.
    ///
    /// In contrast to `api::r0::session::login::call()`, this method stores the
    /// session data returned by the endpoint in this client, instead of
    /// returning it.
    pub async fn log_in(
        &self,
        user: String,
        password: String,
        device_id: Option<Box<DeviceId>>,
        initial_device_display_name: Option<String>,
    ) -> Result<Session, Error<api::Error>> {
        use api::r0::session::login;

        let response = self
            .request(login::Request {
                user: login::UserInfo::MatrixId(user),
                login_info: login::LoginInfo::Password { password },
                device_id,
                initial_device_display_name,
            })
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
    pub async fn register_guest(&self) -> Result<Session, Error<api::r0::uiaa::UiaaResponse>> {
        use api::r0::account::register;

        let response = self
            .request(register::Request {
                auth: None,
                device_id: None,
                inhibit_login: false,
                initial_device_display_name: None,
                kind: Some(register::RegistrationKind::Guest),
                password: None,
                username: None,
            })
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
        username: Option<String>,
        password: String,
    ) -> Result<Session, Error<api::r0::uiaa::UiaaResponse>> {
        use api::r0::account::register;

        let response = self
            .request(register::Request {
                auth: None,
                device_id: None,
                inhibit_login: false,
                initial_device_display_name: None,
                kind: Some(register::RegistrationKind::User),
                password: Some(password),
                username,
            })
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
    ///
    /// If the since parameter is None, the first Item might take a significant time to arrive and
    /// be deserialized, because it contains all events that have occurred in the whole lifetime of
    /// the logged-in users account and are visible to them.
    pub fn sync(
        &self,
        filter: Option<api::r0::sync::sync_events::Filter>,
        since: Option<String>,
        set_presence: ruma_common::presence::PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<api::r0::sync::sync_events::Response, Error<api::Error>>>
           + TryStream<Ok = api::r0::sync::sync_events::Response, Error = Error<api::Error>> {
        use api::r0::sync::sync_events;

        let client = self.clone();
        stream::try_unfold(since, move |since| {
            let client = client.clone();
            let filter = filter.clone();

            async move {
                let response = client
                    .request(sync_events::Request {
                        filter,
                        since,
                        full_state: false,
                        set_presence,
                        timeout,
                    })
                    .await?;

                let next_batch_clone = response.next_batch.clone();
                Ok(Some((response, Some(next_batch_clone))))
            }
        })
    }

    /// Makes a request to a Matrix API endpoint.
    pub fn request<Request: Endpoint>(
        &self,
        request: Request,
    ) -> impl Future<Output = Result<Request::Response, Error<Request::ResponseError>>> {
        self.request_with_url_params(request, None)
    }

    /// Makes a request to a Matrix API endpoint including additional URL parameters.
    pub fn request_with_url_params<Request: Endpoint>(
        &self,
        request: Request,
        params: Option<BTreeMap<String, String>>,
    ) -> impl Future<Output = Result<Request::Response, Error<Request::ResponseError>>> {
        let client = self.0.clone();

        let mut url = client.homeserver_url.clone();

        async move {
            let mut hyper_request = request.try_into()?.map(hyper::Body::from);

            {
                let uri = hyper_request.uri();

                url.set_path(uri.path());
                url.set_query(uri.query());

                if let Some(params) = params {
                    for (key, value) in params {
                        url.query_pairs_mut().append_pair(&key, &value);
                    }
                }

                if Request::METADATA.requires_authentication {
                    if let Some(ref session) = *client.session.lock().unwrap() {
                        url.query_pairs_mut()
                            .append_pair("access_token", &session.access_token);
                    } else {
                        return Err(Error::AuthenticationRequired);
                    }
                }
            }

            *hyper_request.uri_mut() = Uri::from_str(url.as_ref())?;

            let hyper_response = client.hyper.request(hyper_request).await?;
            let (head, body) = hyper_response.into_parts();

            // FIXME: We read the response into a contiguous buffer here (not actually required for
            // deserialization) and then copy the whole thing to convert from Bytes to Vec<u8>.
            let full_body = hyper::body::to_bytes(body).await?;
            let full_response = HttpResponse::from_parts(head, full_body.as_ref().to_owned());

            Ok(Request::Response::try_from(full_response)?)
        }
    }
}

impl<C> Clone for Client<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
