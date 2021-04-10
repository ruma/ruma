use ruma_api::{ruma_api, OutgoingRequest as _, OutgoingResponse as _};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "no_fields",
        path: "/_matrix/my/endpoint",
        rate_limited: false,
        authentication: None,
    }

    request: {}
    response: {}
}

#[test]
fn empty_request_http_repr() {
    let req = Request {};
    let http_req = req.try_into_http_request("https://homeserver.tld", None).unwrap();

    assert!(http_req.body().is_empty());
}

#[test]
fn empty_response_http_repr() {
    let res = Response {};
    let http_res = res.try_into_http_response().unwrap();

    assert_eq!(http_res.body(), b"{}");
}
