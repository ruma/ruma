use std::convert::TryFrom;

use ruma_api::{
    error::{FromHttpResponseError, IntoHttpError, Void},
    ruma_api,
};
use ruma_serde::Outgoing;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        path: "/_matrix/some/endpoint/:baz",
        rate_limited: false,
        authentication: None,
    }

    #[derive(PartialEq)] // Make sure attributes work
    request: {
        // With no attribute on the field, it will be put into the body of the request.
        pub foo: String,
    }
}

#[derive(Outgoing)]
pub struct Response;

impl TryFrom<http::Response<Vec<u8>>> for Response {
    type Error = FromHttpResponseError<Void>;

    fn try_from(_: http::Response<Vec<u8>>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<Response> for http::Response<Vec<u8>> {
    type Error = IntoHttpError;

    fn try_from(_: Response) -> Result<Self, Self::Error> {
        todo!()
    }
}

fn main() {
    let req1 = Request { foo: "foo".into() };
    let req2 = req1.clone();

    assert_eq!(req1, req2);
}
