use ruma_common::api::{ruma_api, IncomingRequest as _};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "some_path_args",
        unstable_path: "/_matrix/:one/a/:two/b/:three/c",
        rate_limited: false,
        authentication: None,
    }

    request: {
        #[ruma_api(path)]
        pub three: String,

        #[ruma_api(path)]
        pub one: String,

        #[ruma_api(path)]
        pub two: String,
    }

    response: {}
}

#[test]
fn path_ordering_is_correct() {
    let request = http::Request::builder()
        .method("GET")
        // This explicitly puts wrong values in the URI, as now we rely on the side-supplied
        // path_args slice, so this is just to ensure it *is* using that slice.
        .uri("https://www.rust-lang.org/_matrix/non/a/non/b/non/c")
        .body("")
        .unwrap();

    let resp = Request::try_from_http_request(request, &["1", "2", "3"]).unwrap();

    assert_eq!(resp.one, "1");
    assert_eq!(resp.two, "2");
    assert_eq!(resp.three, "3");
}
