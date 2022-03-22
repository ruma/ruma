use bytes::BufMut;
use ruma_common::api::{
    error::{FromHttpResponseError, IntoHttpError, MatrixError},
    ruma_api, IncomingResponse, OutgoingResponse,
};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        unstable_path: "/_matrix/some/endpoint/:foo",
        rate_limited: false,
        authentication: None,
    }

    #[derive(PartialEq)] // Make sure attributes work
    request: {
        // With no attribute on the field, it will be put into the body of the request.
        #[ruma_api(path)]
        pub foo: String,
    }
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
