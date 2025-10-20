#![allow(clippy::exhaustive_structs)]

use std::borrow::Cow;

use http::header::{Entry, CONTENT_TYPE, LOCATION};
use ruma_common::{
    api::{
        auth_scheme::{NoAuthentication, SendAccessToken},
        request, response, MatrixVersion, OutgoingRequest as _, OutgoingResponse as _,
        SupportedVersions,
    },
    metadata,
};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAuthentication,
    history: {
        unstable => "/_matrix/my/endpoint",
    }
}

/// Request type for the `no_fields` endpoint.
#[request]
pub struct Request {
    #[ruma_api(header = LOCATION)]
    pub location: Option<String>,

    #[ruma_api(header = CONTENT_TYPE)]
    pub stuff: String,
}

/// Response type for the `no_fields` endpoint.
#[response]
pub struct Response {
    #[ruma_api(header = CONTENT_TYPE)]
    pub stuff: String,
}

#[test]
fn response_content_type_override() {
    let res = Response { stuff: "magic".into() };
    let mut http_res = res.try_into_http_response::<Vec<u8>>().unwrap();

    // Test that we correctly replaced the default content type,
    // not adding another content-type header.
    assert_eq!(
        match http_res.headers_mut().entry(CONTENT_TYPE) {
            Entry::Occupied(occ) => occ.iter().count(),
            _ => 0,
        },
        1
    );
    assert_eq!(http_res.headers().get("content-type").unwrap(), "magic");
}

#[test]
fn request_content_type_override() {
    let req = Request { location: None, stuff: "magic".into() };
    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Default::default() };

    let mut http_req = req
        .try_into_http_request::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            Cow::Owned(supported),
        )
        .unwrap();

    assert_eq!(
        match http_req.headers_mut().entry(CONTENT_TYPE) {
            Entry::Occupied(occ) => occ.iter().count(),
            _ => 0,
        },
        1
    );
    assert_eq!(http_req.headers().get("content-type").unwrap(), "magic");
}
