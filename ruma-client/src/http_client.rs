//! This module contains an abstraction for HTTP clients as well as friendly-named re-exports of
//! client types that implement this trait.

use async_trait::async_trait;
use bytes::BufMut;

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
    type RequestBody: Default + BufMut;

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
