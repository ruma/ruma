#![allow(unexpected_cfgs)]

use bytes::BufMut;
use ruma_common::{
    api::{
        error::{FromHttpResponseError, IntoHttpError, MatrixError},
        request, IncomingResponse, Metadata, OutgoingResponse,
    },
    metadata,
};

const METADATA: Metadata = metadata! {
    method: POST, // An `http::Method` constant. No imports required.
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/some/endpoint/{foo}",
    }
};

#[request]
#[derive(PartialEq)] // Make sure attributes work
pub struct Request {
    // With no attribute on the field, it will be put into the body of the request.
    #[ruma_api(path)]
    pub foo: String,
}

pub struct Response;

impl IncomingResponse for Response {
    type EndpointError = MatrixError;

    fn try_from_http_response<T: AsRef<[u8]>>(
        _: http::Response<T>,
    ) -> Result<Self, FromHttpResponseError<MatrixError>> {
        todo!()
    }
}

impl OutgoingResponse for Response {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        todo!()
    }
}

fn main() {
    let req1 = Request { foo: "foo".into() };
    let req2 = req1.clone();

    assert_eq!(req1, req2);
}
