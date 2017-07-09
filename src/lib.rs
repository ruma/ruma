//! Crate `ruma_client` is a [Matrix](https://matrix.org/) client library.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![feature(conservative_impl_trait, try_from)]

extern crate futures;
extern crate hyper;
#[cfg(feature = "tls")]
extern crate hyper_tls;
#[cfg(feature = "tls")]
extern crate native_tls;
extern crate ruma_api;
extern crate ruma_client_api;
extern crate ruma_identifiers;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate url;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;
use std::str::FromStr;

use futures::future::{Future, FutureFrom, IntoFuture};
use hyper::{Client as HyperClient, Uri};
use hyper::client::{Connect, HttpConnector};
#[cfg(feature = "hyper-tls")]
use hyper_tls::HttpsConnector;
#[cfg(feature = "hyper-tls")]
use native_tls::Error as NativeTlsError;
use ruma_api::Endpoint;
use tokio_core::reactor::Handle;
use url::Url;

use api::r0::session::login;
pub use error::Error;
pub use session::Session;

/// Matrix client-server API endpoints.
pub mod api;
mod error;
mod session;

/// A client for the Matrix client-server API.
#[derive(Clone, Debug)]
pub struct Client<C>
where
    C: Connect,
{
    homeserver_url: Url,
    hyper: Rc<HyperClient<C>>,
    session: RefCell<Option<Session>>,
}

impl Client<HttpConnector> {
    /// Creates a new client for making HTTP requests to the given homeserver.
    pub fn new(handle: &Handle, homeserver_url: Url, session: Option<Session>) -> Self {
        Client {
            homeserver_url,
            hyper: Rc::new(HyperClient::configure().keep_alive(true).build(handle)),
            session: RefCell::new(session),
        }
    }
}

#[cfg(feature = "tls")]
impl Client<HttpsConnector<HttpConnector>> {
    /// Creates a new client for making HTTPS requests to the given homeserver.
    pub fn https(handle: &Handle, homeserver_url: Url, session: Option<Session>) -> Result<Self, NativeTlsError> {
        let connector = HttpsConnector::new(4, handle)?;

        Ok(Client {
            homeserver_url,
            hyper: Rc::new(
                HyperClient::configure()
                    .connector(connector)
                    .keep_alive(true)
                    .build(handle),
            ),
            session: RefCell::new(session),
        })
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
        Client {
            homeserver_url,
            hyper: Rc::new(hyper_client),
            session: RefCell::new(session),
        }
    }

    /// Log in with a username and password.
    ///
    /// In contrast to api::r0::session::login::call(), this method stores the
    /// session data returned by the endpoint in this client, instead of
    /// returning it.
    pub fn log_in<'a>(&'a self, user: String, password: String)
    -> impl Future<Item = (), Error = Error> + 'a {
        let request = login::Request {
            address: None,
            login_type: login::LoginType::Password,
            medium: None,
            password,
            user,
        };

        login::call(self, request).and_then(move |response| {
            *self.session.borrow_mut() = Some(Session::new(response.access_token, response.user_id));

            Ok(())
        })
    }

    /// Register as a guest. In contrast to api::r0::account::register::call(),
    /// this method stores the session data returned by the endpoint in this
    /// client, instead of returning it.
    pub fn register_guest<'a>(&'a self) -> impl Future<Item = (), Error = Error> + 'a {
        use api::r0::account::register;

        register::call(self, register::Request {
            auth: None,
            bind_email: None,
            device_id: None,
            initial_device_display_name: None,
            kind: Some(register::RegistrationKind::Guest),
            password: None,
            username: None,
        }).map(move |response| {
            *self.session.borrow_mut() =
                Some(Session::new(response.access_token, response.user_id));
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
    pub fn register_user<'a>(
        &'a self,
        username: Option<String>,
        password: String,
    ) -> impl Future<Item = (), Error = Error> + 'a {
        use api::r0::account::register;

        register::call(self, register::Request {
            auth: None,
            bind_email: None,
            device_id: None,
            initial_device_display_name: None,
            kind: Some(register::RegistrationKind::User),
            password: Some(password),
            username: username,
        }).map(move |response| {
            *self.session.borrow_mut() =
                Some(Session::new(response.access_token, response.user_id));
        })
    }

    /// Makes a request to a Matrix API endpoint.
    pub(crate) fn request<'a, E>(
        &'a self,
        request: <E as Endpoint>::Request,
    ) -> impl Future<Item = E::Response, Error = Error> + 'a
    where
        E: Endpoint,
        <E as Endpoint>::Response: 'a,
    {
        let mut url = self.homeserver_url.clone();

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
                        if let Some(ref session) = *self.session.borrow() {
                            url.query_pairs_mut().append_pair("access_token", session.access_token());
                        } else {
                            return Err(Error::AuthenticationRequired);
                        }
                    }
                }

                Uri::from_str(url.as_ref())
                    .map(move |uri| (uri, hyper_request))
                    .map_err(Error::from)
            })
            .and_then(move |(uri, mut hyper_request)| {
                hyper_request.set_uri(uri);

                self.hyper
                    .clone()
                    .request(hyper_request)
                    .map_err(Error::from)
            })
            .and_then(|hyper_response| {
                E::Response::future_from(hyper_response).map_err(Error::from)
            })
    }
}
