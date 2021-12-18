use ruma_api::{OutgoingRequest as _, OutgoingResponse as _, SendAccessToken};

mod get {
    ruma_api::ruma_api! {
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
}

mod post {
    ruma_api::ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "no_fields",
            path: "/_matrix/my/endpoint",
            rate_limited: false,
            authentication: None,
        }

        request: {}
        response: {}
    }
}

#[test]
fn empty_post_request_http_repr() {
    let req = post::Request {};
    let http_req = req
        .try_into_http_request::<Vec<u8>>("https://homeserver.tld", SendAccessToken::None)
        .unwrap();

    // Empty POST requests should contain an empty dictionary as a body...
    assert_eq!(http_req.body(), b"{}");
}
#[test]
fn empty_get_request_http_repr() {
    let req = get::Request {};
    let http_req = req
        .try_into_http_request::<Vec<u8>>("https://homeserver.tld", SendAccessToken::None)
        .unwrap();

    // ... but GET requests' bodies should be empty.
    assert!(http_req.body().is_empty());
}

#[test]
fn empty_post_response_http_repr() {
    let res = post::Response {};
    let http_res = res.try_into_http_response::<Vec<u8>>().unwrap();

    // For the reponse, the body should be an empty dict again...
    assert_eq!(http_res.body(), b"{}");
}

#[test]
fn empty_get_response_http_repr() {
    let res = get::Response {};
    let http_res = res.try_into_http_response::<Vec<u8>>().unwrap();

    // ... even for GET requests.
    assert_eq!(http_res.body(), b"{}");
}
