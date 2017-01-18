//! Crate `ruma_client` is a [Matrix](https://matrix.org/) client library.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

extern crate futures;
extern crate hyper;
extern crate ruma_client_api;
extern crate ruma_identifiers;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate url;

use std::borrow::Cow;
use std::fmt::Debug;

use hyper::client::{Client as HyperClient, HttpConnector, Request as HyperRequest};
use hyper::Method as HyperMethod;
use ruma_client_api::{Endpoint, Method};
use ruma_client_api::unversioned::get_supported_versions;
use tokio_core::reactor::Handle;
use url::Url;

pub use error::Error;
pub use session::Session;
pub use response::{FutureResponse, Response};

mod error;
mod response;
mod session;

/// A client for the Matrix client-server API.
#[derive(Debug)]
pub struct Client {
    homeserver_url: Url,
    hyper: HyperClient<HttpConnector>,
    session: Option<Session>,
}

trait IntoHyperMethod {
    fn into_hyper(self) -> HyperMethod;
}

impl IntoHyperMethod for Method {
    fn into_hyper(self) -> HyperMethod {
        match self {
            Method::Delete => HyperMethod::Delete,
            Method::Get => HyperMethod::Get,
            Method::Put => HyperMethod::Put,
            Method::Post => HyperMethod::Post,
        }
    }
}

impl Client {
    /// Creates a new client for making requests to the given homeserver.
    ///
    /// # Errors
    ///
    /// Returns an error if the given homeserver URL cannot be parsed as a URL.
    pub fn new<U>(handle: &Handle, homeserver_url: U) -> Result<Self, Error> where U: TryIntoUrl {
        Ok(Client {
            homeserver_url: homeserver_url.try_into()?,
            hyper: HyperClient::configure().keep_alive(true).build(handle),
            session: None,
        })
    }

    /// Get the versions of the Matrix client-server specification supported by the homeserver.
    pub fn get_supported_versions(&mut self)
    -> Result<FutureResponse<<get_supported_versions::Endpoint as Endpoint>::Response>, Error> {
        self.request::<get_supported_versions::Endpoint>(None, None, None)
    }

    fn request<E>(
        &mut self,
        body_params: Option<E::BodyParams>,
        path_params: Option<E::PathParams>,
        query_params: Option<E::QueryParams>,
    ) -> Result<FutureResponse<E::Response>, Error>
    where E: Endpoint, <E as Endpoint>::Response: Debug + Send  {
        let path = match path_params {
            Some(params) => Cow::from(E::request_path(params)),
            None => Cow::from(E::router_path()),
        };

        let mut url = self.homeserver_url.join(path.as_ref())?.try_into()?;

        if let Some(params) = query_params {
            url.set_query(Some(&serde_urlencoded::to_string(&params)?));
        }

        if E::requires_authentication() {
            if let Some(ref session) = self.session {
                url.query_pairs_mut().append_pair("access_token", &session.access_token);
            } else {
                return Err(Error::AuthenticationRequired);
            }
        }

        let mut request = HyperRequest::new(E::method().into_hyper(), url);

        match E::method() {
            Method::Post | Method::Put => {
                if let Some(params) = body_params {
                    request.set_body(serde_json::to_string(&params)?);
                }
            }
            _ => {}
        }

        Ok(FutureResponse::from(self.hyper.request(request)))
    }
}

/// Functionally equivalent to `TryInto<Url>`, and should be replaced by that as soon as it's
/// stable and available.
pub trait TryIntoUrl {
    /// Performs the conversion.
    fn try_into(self) -> Result<Url, Error>;
}

impl TryIntoUrl for String {
    fn try_into(self) -> Result<Url, Error> {
        Url::parse(&self).map_err(Error::from)
    }
}

impl<'a> TryIntoUrl for &'a str {
    fn try_into(self) -> Result<Url, Error> {
        Url::parse(self).map_err(Error::from)
    }
}

impl TryIntoUrl for Url {
    fn try_into(self) -> Result<Url, Error> {
        Ok(self)
    }
}
