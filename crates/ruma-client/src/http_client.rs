//! This module contains an abstraction for HTTP clients as well as friendly-named re-exports of
//! client types that implement this trait.

use std::{future::Future, pin::Pin};

use bytes::BufMut;
use ruma_common::{
    api::{MatrixVersion, OutgoingRequest, SendAccessToken},
    UserId,
};

use crate::{add_user_id_to_query, ResponseError, ResponseResult};

#[cfg(feature = "hyper")]
mod hyper;
#[cfg(feature = "reqwest")]
mod reqwest;

#[cfg(feature = "hyper")]
pub use self::hyper::Hyper;
#[cfg(feature = "hyper-native-tls")]
pub use self::hyper::HyperNativeTls;
#[cfg(feature = "hyper-rustls")]
pub use self::hyper::HyperRustls;
#[cfg(feature = "reqwest")]
pub use self::reqwest::Reqwest;

/// An HTTP client that can be used to send requests to a Matrix homeserver.
pub trait HttpClient: Sync {
    /// The type to use for `try_into_http_request`.
    type RequestBody: Default + BufMut + Send;

    /// The type to use for `try_from_http_response`.
    type ResponseBody: AsRef<[u8]>;

    /// The error type for the `send_request` function.
    type Error: Send + Unpin;

    /// Send an `http::Request` to get back an `http::Response`.
    fn send_http_request(
        &self,
        req: http::Request<Self::RequestBody>,
    ) -> impl Future<Output = Result<http::Response<Self::ResponseBody>, Self::Error>> + Send;
}

/// An HTTP client that has a default configuration.
pub trait DefaultConstructibleHttpClient: HttpClient {
    /// Creates a new HTTP client with default configuration.
    fn default() -> Self;
}

/// Convenience functionality on top of `HttpClient`.
///
/// If you want to build your own matrix client type instead of using `ruma_client::Client`, this
/// trait should make that relatively easy.
pub trait HttpClientExt: HttpClient {
    /// Send a strongly-typed matrix request to get back a strongly-typed response.
    // TODO: `R: 'a` bound should not be needed
    fn send_matrix_request<'a, R: OutgoingRequest + 'a>(
        &'a self,
        homeserver_url: &str,
        access_token: SendAccessToken<'_>,
        for_versions: &[MatrixVersion],
        request: R,
    ) -> Pin<Box<dyn Future<Output = ResponseResult<Self, R>> + 'a + Send>> {
        self.send_customized_matrix_request(
            homeserver_url,
            access_token,
            for_versions,
            request,
            |_| Ok(()),
        )
    }

    /// Turn a strongly-typed matrix request into an `http::Request`, customize it and send it to
    /// get back a strongly-typed response.
    // TODO: `R: 'a` and `F: 'a` should not be needed
    fn send_customized_matrix_request<'a, R, F>(
        &'a self,
        homeserver_url: &str,
        access_token: SendAccessToken<'_>,
        for_versions: &[MatrixVersion],
        request: R,
        customize: F,
    ) -> Pin<Box<dyn Future<Output = ResponseResult<Self, R>> + 'a + Send>>
    where
        R: OutgoingRequest + 'a,
        F: FnOnce(&mut http::Request<Self::RequestBody>) -> Result<(), ResponseError<Self, R>> + 'a,
    {
        Box::pin(crate::send_customized_request(
            self,
            homeserver_url,
            access_token,
            for_versions,
            request,
            customize,
        ))
    }

    /// Turn a strongly-typed matrix request into an `http::Request`, add a `user_id` query
    /// parameter to it and send it to get back a strongly-typed response.
    ///
    /// This method is meant to be used by application services when interacting with the
    /// client-server API.
    fn send_matrix_request_as<'a, R: OutgoingRequest + 'a>(
        &'a self,
        homeserver_url: &str,
        access_token: SendAccessToken<'_>,
        for_versions: &[MatrixVersion],
        user_id: &'a UserId,
        request: R,
    ) -> Pin<Box<dyn Future<Output = ResponseResult<Self, R>> + 'a>> {
        self.send_customized_matrix_request(
            homeserver_url,
            access_token,
            for_versions,
            request,
            add_user_id_to_query::<Self, R>(user_id),
        )
    }
}

impl<T: HttpClient> HttpClientExt for T {}

#[doc(hidden)]
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Dummy;

impl HttpClient for Dummy {
    type RequestBody = Vec<u8>;
    type ResponseBody = Vec<u8>;
    type Error = ();

    #[allow(clippy::diverging_sub_expression)]
    async fn send_http_request(
        &self,
        _req: http::Request<Self::RequestBody>,
    ) -> Result<http::Response<Self::ResponseBody>, Self::Error> {
        unimplemented!("this client only exists to allow doctests to compile")
    }
}

impl DefaultConstructibleHttpClient for Dummy {
    fn default() -> Self {
        Dummy
    }
}
