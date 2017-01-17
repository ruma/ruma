use std::fmt::Debug;
use std::io::Write;
use std::marker::PhantomData;

use futures::{Async, Future, Poll, Stream};
use hyper::client::{FutureResponse as HyperFutureResponse, Response as HyperResponse};
use hyper::Error as HyperError;
use hyper::header::Headers;
use hyper::status::StatusCode;
use hyper::HttpVersion;
use serde::Deserialize;
use serde_json::from_slice;

use Error;

/// A `Future` that will resolve into a `Response`.
#[derive(Debug)]
pub struct FutureResponse<T> where T: Debug + Deserialize + Send + 'static {
    hyper_future_response: HyperFutureResponse,
    phantom: PhantomData<T>,
}

/// A response from a Matrix homeserver.
#[derive(Debug)]
pub struct Response<T> where T: Debug + Deserialize {
    /// The Hyper response.
    hyper_response: HyperResponse,
    /// The response from the Matrix API.
    phantom: PhantomData<T>,
}

impl<T> Future for FutureResponse<T> where T: Debug + Deserialize + Send + 'static {
    type Item = Response<T>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.hyper_future_response.poll() {
            Ok(Async::Ready(hyper_response)) => Ok(Async::Ready(Response {
                hyper_response: hyper_response,
                phantom: PhantomData,
            })),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(Error::from(error)),
        }
    }
}

impl<T> From<HyperFutureResponse> for FutureResponse<T>
where T: Debug + Deserialize + Send + 'static {
    fn from(hyper_future_response: HyperFutureResponse) -> FutureResponse<T> {
        FutureResponse {
            hyper_future_response: hyper_future_response,
            phantom: PhantomData,
        }
    }
}

impl<T> Response<T> where T: Debug + Deserialize + Send + 'static {
    /// The response from the Matrix API.
    pub fn data(self) -> Box<Future<Item=T, Error=Error>> {
        let bytes = self.hyper_response.body().fold(Vec::new(), |mut bytes, chunk| {
            if let Err(error) = bytes.write_all(&chunk) {
                return Err(HyperError::from(error));
            }

            Ok(bytes)
        }).map_err(Error::from);

        let deserialized_data = bytes.and_then(|bytes| {
            from_slice(bytes.as_slice()).map_err(Error::from)
        });

        deserialized_data.boxed()
    }

    /// The HTTP response code.
    pub fn status(&self) -> &StatusCode {
        self.hyper_response.status()
    }

    /// The HTTP response headers.
    pub fn headers(&self) -> &Headers {
        self.hyper_response.headers()
    }

    /// The HTTP version.
    pub fn http_version(&self) -> &HttpVersion {
        self.hyper_response.version()
    }
}
