// This tests that the "body" fields are moved after all other fields because they
// consume the request/response.

mod newtype_body {
    use ruma_api::ruma_api;
    use ruma_identifiers::UserId;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(body)]
            pub q2: Foo,

            #[ruma_api(path)]
            pub bar: String,

            #[ruma_api(query)]
            pub baz: UserId,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }

        response: {
            #[ruma_api(body)]
            pub q2: Foo,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }
    }
}

mod newtype_raw_body {
    use ruma_api::ruma_api;
    use ruma_identifiers::UserId;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(raw_body)]
            pub q2: Vec<u8>,

            #[ruma_api(path)]
            pub bar: String,

            #[ruma_api(query)]
            pub baz: UserId,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }

        response: {
            #[ruma_api(raw_body)]
            pub q2: Vec<u8>,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }
    }
}

mod plain {
    use ruma_api::ruma_api;
    use ruma_identifiers::UserId;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            pub q2: Foo,

            pub bar: String,

            #[ruma_api(query)]
            pub baz: UserId,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }

        response: {
            pub q2: Vec<u8>,

            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
        }
    }
}

fn main() {}
