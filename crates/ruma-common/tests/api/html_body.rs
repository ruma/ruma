#![allow(clippy::exhaustive_structs)]

use bytes::BytesMut;
use http::{header::CONTENT_TYPE, HeaderValue};
use ruma_common::{
    api::{request, response, Metadata, OutgoingRequest, SendAccessToken},
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
    #[ruma_api(header = CONTENT_TYPE)]
    content_type: String,

    /// HTML to return to client.
    #[ruma_api(raw_body)]
    pub body: Vec<u8>,
}

/// Response type for the `html_body` endpoint.
#[response]
pub struct Response {}

#[test]
fn response_html_body() {
    let content_type = "text/html; charset=utf-8";

    let req = Request {
        content_type: content_type.to_owned(),
        body: b"
            <p>This is a paragraph.</p>
            <p>This is another paragraph.</p>
        "
        .to_vec(),
    };

    let req: http::Request<BytesMut> =
        req.try_into_http_request("https://homeserver.tld", SendAccessToken::None, &[]).unwrap();

    assert_eq!(
        Some(content_type),
        req.headers().get(CONTENT_TYPE).map(HeaderValue::to_str).transpose().unwrap()
    );
}
