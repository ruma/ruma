use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        path: "/_matrix/foo/:bar/",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        #[ruma_api(body)]
        pub q2: Vec<u8>,

        #[ruma_api(path)]
        pub bar: String,

        #[ruma_api(query)]
        pub baz: UserId,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }

    response: {
        #[ruma_api(body)]
        pub q2: Vec<u8>,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }
}

fn main() {}
