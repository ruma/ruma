use ruma_common::api::ruma_api;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        unstable_path: "/_matrix/some/endpoint/:baz",
        rate_limited: false,
        authentication: None,
    }

    request: {}

    response: {}

    #[derive(Default)]
    error: ruma_common::api::error::MatrixError
}

fn main() {}
