//! Crate `ruma_client` is a [Matrix](https://matrix.org/) client library.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![feature(try_from)]

extern crate futures;
extern crate hyper;
#[cfg(feature = "tls")]
extern crate hyper_tls;
#[cfg(feature = "tls")]
extern crate native_tls;
extern crate ruma_api;
extern crate ruma_client_api;
extern crate ruma_identifiers;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate url;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;
use std::str::FromStr;

use futures::future::{Future, FutureFrom, IntoFuture};
use futures::stream::{self, Stream};
use hyper::{Client as HyperClient, Uri};
use hyper::client::{Connect, HttpConnector};
#[cfg(feature = "hyper-tls")]
use hyper_tls::HttpsConnector;
#[cfg(feature = "hyper-tls")]
use native_tls::Error as NativeTlsError;
use ruma_api::Endpoint;
use tokio_core::reactor::Handle;
use url::Url;

pub use error::Error;
pub use session::Session;

/// Matrix client-server API endpoints.
pub mod api;
mod error;
mod session;

/// A client for the Matrix client-server API.
#[derive(Debug)]
pub struct Client<C: Connect>(Rc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
pub struct ClientData<C>
where
    C: Connect,
{
    homeserver_url: Url,
    hyper: HyperClient<C>,
    session: RefCell<Option<Session>>,
}

impl Client<HttpConnector> {
    /// Creates a new client for making HTTP requests to the given homeserver.
    pub fn new(handle: &Handle, homeserver_url: Url, session: Option<Session>) -> Self {
        Client(Rc::new(ClientData {
            homeserver_url: homeserver_url,
            hyper: HyperClient::configure().keep_alive(true).build(handle),
            session: RefCell::new(session),
        }))
    }
}

#[cfg(feature = "tls")]
impl Client<HttpsConnector<HttpConnector>> {
    /// Creates a new client for making HTTPS requests to the given homeserver.
    pub fn https(handle: &Handle, homeserver_url: Url, session: Option<Session>) -> Result<Self, NativeTlsError> {
        let connector = HttpsConnector::new(4, handle)?;

        Ok(Client(Rc::new(ClientData {
            homeserver_url: homeserver_url,
            hyper: {
                HyperClient::configure()
                    .connector(connector)
                    .keep_alive(true)
                    .build(handle)
            },
            session: RefCell::new(session),
        })))
    }
}

impl<C> Client<C>
where
    C: Connect,
{
    /// Creates a new client using the given `hyper::Client`.
    ///
    /// This allows the user to configure the details of HTTP as desired.
    pub fn custom(hyper_client: HyperClient<C>, homeserver_url: Url, session: Option<Session>) -> Self {
        Client(Rc::new(ClientData {
            homeserver_url: homeserver_url,
            hyper: hyper_client,
            session: RefCell::new(session),
        }))
    }

    /// Log in with a username and password.
    ///
    /// In contrast to api::r0::session::login::call(), this method stores the
    /// session data returned by the endpoint in this client, instead of
    /// returning it.
    pub fn log_in(&self, user: String, password: String)
    -> impl Future<Item = Session, Error = Error> {
        use api::r0::session::login;

        let data = self.0.clone();

        login::call(self.clone(), login::Request {
            address: None,
            login_type: login::LoginType::Password,
            medium: None,
            password,
            user,
        }).map(move |response| {
            let session = Session::new(response.access_token, response.user_id);
            *data.session.borrow_mut() = Some(session.clone());

            session
        })
    }

    /// Register as a guest. In contrast to api::r0::account::register::call(),
    /// this method stores the session data returned by the endpoint in this
    /// client, instead of returning it.
    pub fn register_guest(&self) -> impl Future<Item = Session, Error = Error> {
        use api::r0::account::register;

        let data = self.0.clone();

        register::call(self.clone(), register::Request {
            auth: None,
            bind_email: None,
            device_id: None,
            initial_device_display_name: None,
            kind: Some(register::RegistrationKind::Guest),
            password: None,
            username: None,
        }).map(move |response| {
            let session = Session::new(response.access_token, response.user_id);
            *data.session.borrow_mut() = Some(session.clone());

            session
        })
    }

    /// Register as a new user on this server.
    ///
    /// In contrast to api::r0::account::register::call(), this method stores
    /// the session data returned by the endpoint in this client, instead of
    /// returning it.
    ///
    /// The username is the local part of the returned user_id. If it is
    /// omitted from this request, the server will generate one.
    pub fn register_user(
        &self,
        username: Option<String>,
        password: String,
    ) -> impl Future<Item = Session, Error = Error> {
        use api::r0::account::register;

        let data = self.0.clone();

        register::call(self.clone(), register::Request {
            auth: None,
            bind_email: None,
            device_id: None,
            initial_device_display_name: None,
            kind: Some(register::RegistrationKind::User),
            password: Some(password),
            username: username,
        }).map(move |response| {
            let session = Session::new(response.access_token, response.user_id);
            *data.session.borrow_mut() = Some(session.clone());

            session
        })
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    ///
    /// If the since parameter is None, the first Item might take a significant time to arrive and
    /// be deserialized, because it contains all events that have occured in the whole lifetime of
    /// the logged-in users account and are visible to them.
    pub fn sync(
        &self,
        filter: Option<api::r0::sync::sync_events::Filter>,
        since: Option<String>,
        set_presence: bool,
    ) -> impl Stream<Item = api::r0::sync::sync_events::Response, Error = Error> {
        use api::r0::sync::sync_events;

        let client = self.clone();
        let set_presence = if set_presence {
            None
        } else {
            Some(sync_events::SetPresence::Offline)
        };

        stream::unfold(since, move |since| {
            Some(
                sync_events::call(
                    client.clone(),
                    sync_events::Request {
                        filter: filter.clone(),
                        since,
                        full_state: None,
                        set_presence: set_presence.clone(),
                        timeout: None,
                    },
                ).map(|res| {
                    let next_batch_clone = res.next_batch.clone();
                    (res, Some(next_batch_clone))
                })
            )
        })
    }

    /// Makes a request to a Matrix API endpoint.
    pub(crate) fn request<E>(
        self,
        request: <E as Endpoint>::Request,
    ) -> impl Future<Item = E::Response, Error = Error>
    where
        E: Endpoint,
    {
        let data1 = self.0.clone();
        let data2 = self.0.clone();
        let mut url = self.0.homeserver_url.clone();

        request
            .try_into()
            .map_err(Error::from)
            .into_future()
            .and_then(move |hyper_request| {
                {
                    let uri = hyper_request.uri();

                    url.set_path(uri.path());
                    url.set_query(uri.query());

                    if E::METADATA.requires_authentication {
                        if let Some(ref session) = *data1.session.borrow() {
                            url.query_pairs_mut().append_pair("access_token", session.access_token());
                        } else {
                            return Err(Error::AuthenticationRequired);
                        }
                    }
                }

                Uri::from_str(url.as_ref().as_ref())
                    .map(move |uri| (uri, hyper_request))
                    .map_err(Error::from)
            })
            .and_then(move |(uri, mut hyper_request)| {
                hyper_request.set_uri(uri);

                data2.hyper
                    .request(hyper_request)
                    .map_err(Error::from)
            })
            .and_then(|hyper_response| {
                E::Response::future_from(hyper_response).map_err(Error::from)
            })
    }
}

impl<C: Connect> Clone for Client<C> {
    fn clone(&self) -> Client<C> {
        Client(self.0.clone())
    }
}
