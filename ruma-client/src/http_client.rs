//! This module contains an abstraction for HTTP clients as well as friendly-named re-exports of
//! client types that implement this trait.

use async_trait::async_trait;
use bytes::BufMut;
use ruma_api::{OutgoingRequest, SendAccessToken};

use crate::Error;

#[cfg(feature = "hyper")]
mod hyper;

#[cfg(feature = "hyper")]
pub use self::hyper::Hyper;
#[cfg(feature = "hyper-native-tls")]
pub use self::hyper::HyperNativeTls;
#[cfg(feature = "hyper-rustls")]
pub use self::hyper::HyperRustls;

/// An HTTP client that can be used to send requests to a Matrix homeserver.
#[async_trait]
pub trait HttpClient {
    /// The type to use for `try_into_http_request`.
    type RequestBody: Default + BufMut + Send;

    /// The type to use for `try_from_http_response`.
    type ResponseBody: AsRef<[u8]>;

    /// The error type for the `send_request` function.
    type Error: Unpin;

    /// Send an `http::Request` to get back an `http::Response`.
    async fn send_http_request(
        &self,
        req: http::Request<Self::RequestBody>,
    ) -> Result<http::Response<Self::ResponseBody>, Self::Error>;
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
#[async_trait]
pub trait HttpClientExt: HttpClient {
    /// Send a strongly-typed matrix request to get back a strongly-typed response.
    async fn send_request<Request: OutgoingRequest>(
        &self,
        homeserver_url: &str,
        access_token: SendAccessToken<'_>,
        request: Request,
    ) -> Result<Request::IncomingResponse, Error<Self::Error, Request::EndpointError>> {
        crate::send_request_with_url_params(self, homeserver_url, access_token, None, request).await
    }
}

#[async_trait]
impl<T: HttpClient> HttpClientExt for T {}
