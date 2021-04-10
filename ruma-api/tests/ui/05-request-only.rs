use bytes::Buf;
use ruma_api::{
    error::{FromHttpResponseError, IntoHttpError, Void},
    ruma_api, IncomingResponse, OutgoingResponse,
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

impl IncomingResponse for Response {
    type EndpointError = Void;

    fn try_from_http_response<T: Buf>(
        _: http::Response<T>,
    ) -> Result<Self, FromHttpResponseError<Void>> {
        todo!()
    }
}

impl OutgoingResponse for Response {
    fn try_into_http_response(self) -> Result<http::Response<Vec<u8>>, IntoHttpError> {
        todo!()
    }
}

fn main() {
    let req1 = Request { foo: "foo".into() };
    let req2 = req1.clone();

    assert_eq!(req1, req2);
}
