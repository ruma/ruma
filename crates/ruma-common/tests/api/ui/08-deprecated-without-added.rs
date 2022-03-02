use ruma_common::api::ruma_api;

ruma_api! {
    metadata: {
        description: "This will fail.",
        method: GET,
        name: "invalid_versions",
        unstable_path: "/a/path",
        rate_limited: false,
        authentication: None,

        deprecated: 1.1,
    }

    request: {
        #[ruma_api(query_map)]
        pub fields: Vec<(String, String)>,
    }

    response: {}
}

fn main() {}
