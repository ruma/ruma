use std::convert::TryFrom;
use std::fmt::Debug;

use hyper::client::Response as HyperResponse;
use hyper::header::Headers;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use serde::Deserialize;
use serde_json::from_reader;
use url::Url;

use Error;

/// A response from a Matrix homeserver.
#[derive(Debug)]
pub struct Response<T> where T: Debug + Deserialize {
    /// The response from the Matrix API.
    pub data: T,
    /// The HTTP response code.
    pub status: StatusCode,
    /// The HTTP response headers.
    pub headers: Headers,
    /// The HTTP version.
    pub http_version: HttpVersion,
    /// The URL that was requested.
    pub url: Url,
}

impl<T> TryFrom<HyperResponse> for Response<T> where T: Debug + Deserialize {
    type Err = Error;

    fn try_from(hyper_response: HyperResponse) -> Result<Self, Self::Err> {
        Ok(Response {
            status: hyper_response.status,
            headers: hyper_response.headers.clone(),
            http_version: hyper_response.version,
            url: hyper_response.url.clone(),
            data: from_reader(hyper_response)?,
        })
    }
}
