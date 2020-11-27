use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "This will fail.",
        method: GET,
        name: "invalid_path",
        path: "µ/°/§/€",
        rate_limited: false,
        authentication: None,
    }

    request: {
        #[ruma_api(query_map)]
        pub fields: Vec<(String, String)>,
    }

    response: {}
}

ruma_api! {
    metadata: {
        description: "This will fail.",
        method: GET,
        name: "invalid_path",
        path: "path/to/invalid space/endpoint",
        rate_limited: false,
        authentication: None,
    }

    request: {
        #[ruma_api(query_map)]
        pub fields: Vec<(String, String)>,
    }

    response: {}
}

fn main() {}
