use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "newtype_body_endpoint",
        path: "/_matrix/some/query/map/\u{ffff}",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        #[ruma_api(query_map)]
        pub fields: Vec<(String, String)>,
    }

    response: { }
}

fn main() {}
