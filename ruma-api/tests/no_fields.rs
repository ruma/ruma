use std::convert::TryFrom;

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "no_fields",
        path: "/_matrix/my/endpoint",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {}
    response: {}
}

#[test]
fn empty_request_http_repr() {
    let req = Request {};
    let http_req = http::Request::<Vec<u8>>::try_from(req).unwrap();

    assert!(http_req.body().is_empty());
}

#[test]
fn empty_response_http_repr() {
    let res = Response {};
    let http_res = http::Response::<Vec<u8>>::try_from(res).unwrap();

    assert_eq!(http_res.body(), b"{}");
}
