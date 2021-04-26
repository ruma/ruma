use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use hyper::client::{connect::Connect, HttpConnector};

use super::{DefaultConstructibleHttpClient, HttpClient};

/// A basic hyper HTTP client.
///
/// You basically never want this, since it doesn't support `https`.
pub type Hyper = hyper::Client<HttpConnector>;

/// A hyper HTTP client using native-tls for TLS support.
#[cfg(feature = "hyper-native-tls")]
pub type HyperNativeTls = hyper::Client<hyper_tls::HttpsConnector<HttpConnector>>;

/// A hyper HTTP client using rustls for TLS support.
///
/// This client does not implement `DefaultConstructibleHttpClient`. To use it, you need to manually
/// construct
#[cfg(feature = "hyper-rustls")]
pub type HyperRustls = hyper::Client<hyper_rustls::HttpsConnector<HttpConnector>>;

#[async_trait]
impl<C> HttpClient for hyper::Client<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type RequestBody = BytesMut;
    type ResponseBody = Bytes;
    type Error = hyper::Error;

    async fn send_http_request(
        &self,
        req: http::Request<BytesMut>,
    ) -> Result<http::Response<Bytes>, hyper::Error> {
        let (head, body) = self
            .request(req.map(|body| hyper::body::Body::from(body.freeze())))
            .await?
            .into_parts();

        // FIXME: Use aggregate instead of to_bytes once serde_json can parse from a reader at a
        // comparable speed as reading from a slice: https://github.com/serde-rs/json/issues/160
        let body = hyper::body::to_bytes(body).await?;
        Ok(http::Response::from_parts(head, body))
    }
}

#[cfg(feature = "hyper")]
impl DefaultConstructibleHttpClient for Hyper {
    fn default() -> Self {
        hyper::Client::new()
    }
}

#[cfg(feature = "hyper-native-tls")]
impl DefaultConstructibleHttpClient for HyperNativeTls {
    fn default() -> Self {
        hyper::Client::builder().build(hyper_tls::HttpsConnector::new())
    }
}
