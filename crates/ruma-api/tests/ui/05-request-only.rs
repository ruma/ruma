// #![feature(type_alias_impl_trait)]

use ruma_api::{
    error::{FromHttpResponseError, IntoHttpError, MatrixError},
    ruma_api, IncomingRawHttpBody, IncomingResponse, OutgoingResponse, RawHttpBody,
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
    type IncomingBody = IncomingRawHttpBody;
    type EndpointError = MatrixError;

    fn try_from_http_response(
        _: http::Response<IncomingRawHttpBody>,
    ) -> Result<Self, FromHttpResponseError<MatrixError>> {
        todo!()
    }
}

impl OutgoingResponse for Response {
    type OutgoingBody = RawHttpBody<'static>;

    fn try_into_http_response(self) -> Result<http::Response<RawHttpBody<'static>>, IntoHttpError> {
        todo!()
    }
}

fn main() {
    let req1 = Request { foo: "foo".into() };
    let req2 = req1.clone();

    assert_eq!(req1, req2);
}
