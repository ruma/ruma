#![allow(clippy::exhaustive_structs)]

use http::header::{Entry, CONTENT_TYPE};
use ruma_common::api::{
    ruma_api, MatrixVersion, OutgoingRequest as _, OutgoingResponse as _, SendAccessToken,
};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "no_fields",
        unstable_path: "/_matrix/my/endpoint",
        rate_limited: false,
        authentication: None,
    }

    request: {
        #[ruma_api(header = LOCATION)]
        pub location: Option<String>,

        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: String,
    }

    response: {
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: String,
    }
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
    let mut http_req = req
        .try_into_http_request::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            &[MatrixVersion::V1_1],
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
