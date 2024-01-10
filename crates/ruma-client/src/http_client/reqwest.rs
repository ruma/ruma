use std::mem;

use bytes::{Bytes, BytesMut};

use super::{DefaultConstructibleHttpClient, HttpClient};

/// The `reqwest` crate's `Client`.
pub type Reqwest = reqwest::Client;

impl HttpClient for Reqwest {
    type RequestBody = BytesMut;
    type ResponseBody = Bytes;
    type Error = reqwest::Error;

    async fn send_http_request(
        &self,
        req: http::Request<BytesMut>,
    ) -> Result<http::Response<Bytes>, reqwest::Error> {
        let req = req.map(|body| body.freeze()).try_into()?;
        let mut res = self.execute(req).await?;

        let mut http_builder =
            http::Response::builder().status(res.status()).version(res.version());
        mem::swap(
            http_builder.headers_mut().expect("http::response::Builder to be usable"),
            res.headers_mut(),
        );

        Ok(http_builder.body(res.bytes().await?).expect("http::Response construction to work"))
    }
}

impl DefaultConstructibleHttpClient for Reqwest {
    fn default() -> Self {
        reqwest::Client::new()
    }
}
