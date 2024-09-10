#![allow(clippy::exhaustive_structs)]

use http::StatusCode;
use ruma_common::{
    api::{request, response, Metadata, OutgoingResponse as _},
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

/// Request type for the `default_status` endpoint.
#[request]
pub struct Request {}

/// Response type for the `default_status` endpoint.
#[response]
pub struct Response {}

#[test]
fn response_default_status() {
    let res = Response {};
    let http_res = res.try_into_http_response::<Vec<u8>>().unwrap();

    // Test that we correctly changed the status code.
    assert_eq!(http_res.status(), StatusCode::OK);
}
