#![allow(clippy::exhaustive_structs)]

use bytes::BytesMut;
use http::StatusCode;
use ruma_common::{
    api::{
        request, response, MatrixVersion, Metadata, OutgoingRequest, OutgoingResponse as _,
        SendAccessToken,
    },
    metadata,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/my/endpoint",
    }
};

/// Request type for the `html_body` endpoint.
#[request]
pub struct Request {
    /// HTML to return to client.
    #[ruma_api(raw_body)]
    pub body: Vec<u8>,
}

/// Response type for the `html_body` endpoint.
#[response]
pub struct Response {}

#[test]
fn response_html_body() {
    let body = "
        <p>This is a paragraph.</p>
        <p>This is another paragraph.</p>
    "
    .as_bytes()
    .to_owned();
    let req = Request { body };

    let req: http::Request<BytesMut> = req
        .try_into_http_request("https://foobar.com", SendAccessToken::None, &[MatrixVersion::V1_10])
        .unwrap();

    dbg!(&req.headers());
}
