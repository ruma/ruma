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

    request: {}

    response: {}

    #[derive(Default)]
    error: ruma_api::error::Void
}

fn main() {}
