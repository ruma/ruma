use http::header::{Entry, CONTENT_TYPE};
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
    let mut http_res = res.try_into_http_response().unwrap();

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
    let mut http_req = req.try_into_http_request("https://homeserver.tld", None).unwrap();

    assert_eq!(
        match http_req.headers_mut().entry(CONTENT_TYPE) {
            Entry::Occupied(occ) => occ.iter().count(),
            _ => 0,
        },
        1
    );
    assert_eq!(http_req.headers().get("content-type").unwrap(), "magic");
}
