use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        path: "/_matrix/some/endpoint/:baz",
        rate_limited: false,
        authentication: None,
    }

    #[not_a_real_attribute_should_fail]
    request: {
        pub foo: String,
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,
        #[ruma_api(query)]
        pub bar: String,
        #[ruma_api(path)]
        pub baz: String,
    }

    response: {
        pub value: String,
    }
}

fn main() {}
