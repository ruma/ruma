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

use std::convert::TryInto;
use std::rc::Rc;

use futures::future::{Future, FutureFrom, IntoFuture};
use hyper::Client as HyperClient;
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
pub struct Client<C>
where
    C: Connect,
{
    homeserver_url: Url,
    hyper: Rc<HyperClient<C>>,
    session: Option<Session>,
}

impl Client<HttpConnector> {
    /// Creates a new client for making HTTP requests to the given homeserver.
    pub fn new(handle: &Handle, homeserver_url: Url) -> Self {
        Client {
            homeserver_url,
            hyper: Rc::new(HyperClient::configure().keep_alive(true).build(handle)),
            session: None,
        }
    }
}

#[cfg(feature = "tls")]
impl Client<HttpsConnector<HttpConnector>> {
    /// Creates a new client for making HTTPS requests to the given homeserver.
    pub fn https(handle: &Handle, homeserver_url: Url) -> Result<Self, NativeTlsError> {
        let connector = HttpsConnector::new(4, handle)?;

        Ok(Client {
            homeserver_url,
            hyper: Rc::new(
                HyperClient::configure()
                    .connector(connector)
                    .keep_alive(true)
                    .build(handle),
            ),
            session: None,
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
    pub fn custom(hyper_client: HyperClient<C>, homeserver_url: Url) -> Self {
        Client {
            homeserver_url,
            hyper: Rc::new(hyper_client),
            session: None,
        }
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
        request
            .try_into()
            .map_err(Error::from)
            .into_future()
            .and_then(move |hyper_request| {
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
