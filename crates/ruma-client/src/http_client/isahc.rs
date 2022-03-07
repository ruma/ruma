use async_trait::async_trait;
use futures_lite::AsyncReadExt;

use super::HttpClient;

/// The `isahc` crate's `HttpClient`.
pub type Isahc = isahc::HttpClient;

#[async_trait]
impl HttpClient for Isahc {
    type RequestBody = Vec<u8>;
    type ResponseBody = Vec<u8>;
    type Error = isahc::Error;

    async fn send_http_request(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>, isahc::Error> {
        let (head, mut body) = self.send_async(req).await?.into_parts();
        let mut full_body = Vec::new();
        body.read_to_end(&mut full_body).await?;
        Ok(http::Response::from_parts(head, full_body))
    }
}
