#![allow(clippy::exhaustive_structs)]

use http::{
    header::{Entry, LOCATION},
    StatusCode,
};
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

/// Request type for the `status_override` endpoint.
#[request]
pub struct Request {}

/// Response type for the `status_override` endpoint.
#[response(status = FOUND)]
pub struct Response {
    #[ruma_api(header = LOCATION)]
    pub location: Option<String>,
}

#[test]
fn response_status_override() {
    let res = Response { location: Some("/_matrix/another/endpoint".into()) };
    let mut http_res = res.try_into_http_response::<Vec<u8>>().unwrap();

    // Test that we correctly changed the status code.
    assert_eq!(http_res.status(), StatusCode::FOUND);

    // Test that we correctly replaced the location,
    // not adding another location header.
    assert_eq!(
        match http_res.headers_mut().entry(LOCATION) {
            Entry::Occupied(occ) => occ.iter().count(),
            _ => 0,
        },
        1
    );
    assert_eq!(http_res.headers().get("location").unwrap(), "/_matrix/another/endpoint");
}
