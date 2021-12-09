use ruma_common::api::{
    IntoHttpBody as _, MatrixVersion, OutgoingRequest as _, OutgoingResponse as _, SendAccessToken,
};

mod get {
    use ruma_common::{
        api::{request, response, Metadata},
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

    /// Request type for the `no_fields` endpoint.
    #[request]
    pub struct Request {}

    /// Response type for the `no_fields` endpoint.
    #[response]
    pub struct Response {}
}

mod post {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/my/endpoint",
        }
    };

    /// Request type for the `no_fields` endpoint.
    #[request]
    pub struct Request {}

    /// Response type for the `no_fields` endpoint.
    #[response]
    pub struct Response {}
}

#[test]
fn empty_post_request_http_repr() {
    let req = post::Request {};
    let http_req = req
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::None,
            &[MatrixVersion::V1_1],
        )
        .unwrap();

    // Empty POST requests should contain an empty dictionary as a body...
    assert_eq!(http_req.body().to_buf::<Vec<u8>>().unwrap(), b"{}");
}
#[test]
fn empty_get_request_http_repr() {
    let req = get::Request {};
    let http_req = req
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::None,
            &[MatrixVersion::V1_1],
        )
        .unwrap();

    // ... but GET requests' bodies should be empty.
    assert_eq!(http_req.body().to_buf::<Vec<u8>>().unwrap(), b"");
}

#[test]
fn empty_post_response_http_repr() {
    let res = post::Response {};
    let http_res = res.try_into_http_response().unwrap();

    // For the response, the body should be an empty dict again...
    assert_eq!(http_res.body().to_buf::<Vec<u8>>().unwrap(), b"{}");
}

#[test]
fn empty_get_response_http_repr() {
    let res = get::Response {};
    let http_res = res.try_into_http_response().unwrap();

    // ... even for GET requests.
    assert_eq!(http_res.body().to_buf::<Vec<u8>>().unwrap(), b"{}");
}
