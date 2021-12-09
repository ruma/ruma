// #![feature(type_alias_impl_trait)]

use ruma_api::{
    ruma_api, IntoHttpBody as _, OutgoingRequest as _, OutgoingResponse as _, SendAccessToken,
};

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
    let http_body = req
        .try_into_http_request("https://homeserver.tld", SendAccessToken::None)
        .unwrap()
        .body()
        .to_buf::<Vec<u8>>()
        .unwrap();

    assert!(http_body.is_empty());
}

#[test]
fn empty_response_http_repr() {
    let res = Response {};
    let http_body = res.try_into_http_response().unwrap().body().to_buf::<Vec<u8>>().unwrap();

    assert_eq!(http_body, b"{}");
}
