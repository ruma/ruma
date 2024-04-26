use bytes::{Bytes, BytesMut};
use http_body_util::{BodyExt as _, Full};
use hyper_util::{
    client::legacy::connect::{Connect, HttpConnector},
    rt::TokioExecutor,
};

use super::{DefaultConstructibleHttpClient, HttpClient};

/// A hyper HTTP client.
///
/// The default connector is rarely useful, since it doesn't support `https`.
pub type Hyper<C = HttpConnector> = hyper_util::client::legacy::Client<C, Full<Bytes>>;

/// A hyper HTTP client using native-tls for TLS support.
#[cfg(feature = "hyper-native-tls")]
pub type HyperNativeTls = Hyper<hyper_tls::HttpsConnector<HttpConnector>>;

/// A hyper HTTP client using rustls for TLS support.
///
/// This client does not implement `DefaultConstructibleHttpClient`.
/// To use it, you need to manually create an instance.
#[cfg(feature = "hyper-rustls")]
pub type HyperRustls = Hyper<hyper_rustls::HttpsConnector<HttpConnector>>;

impl<C> HttpClient for Hyper<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type RequestBody = BytesMut;
    type ResponseBody = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn send_http_request(
        &self,
        req: http::Request<BytesMut>,
    ) -> Result<http::Response<Bytes>, Self::Error> {
        let (head, body) =
            self.request(req.map(|body| Full::new(body.freeze()))).await?.into_parts();

        // FIXME: Use aggregate instead of to_bytes once serde_json can parse from a reader at a
        // comparable speed as reading from a slice: https://github.com/serde-rs/json/issues/160
        let body = body.collect().await?.to_bytes();
        Ok(http::Response::from_parts(head, body))
    }
}

#[cfg(feature = "hyper")]
impl DefaultConstructibleHttpClient for Hyper {
    fn default() -> Self {
        hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build(HttpConnector::new())
    }
}

#[cfg(feature = "hyper-native-tls")]
impl DefaultConstructibleHttpClient for HyperNativeTls {
    fn default() -> Self {
        hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build(hyper_tls::HttpsConnector::new())
    }
}
