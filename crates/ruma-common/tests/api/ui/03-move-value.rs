// This tests that the "body" fields are moved after all other fields because they
// consume the request/response.

mod newtype_body {
    use ruma_common::{api::ruma_api, OwnedUserId};

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            unstable_path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(body)]
            pub q2: Foo,

            #[ruma_api(path)]
            pub bar: String,

            #[ruma_api(query)]
            pub baz: OwnedUserId,

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

mod raw_body {
    use ruma_common::{api::ruma_api, OwnedUserId};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            unstable_path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(raw_body)]
            pub q2: Vec<u8>,

            #[ruma_api(path)]
            pub bar: String,

            #[ruma_api(query)]
            pub baz: OwnedUserId,

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
    use ruma_common::{api::ruma_api, OwnedUserId};

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            unstable_path: "/_matrix/foo/:bar/",
            rate_limited: false,
            authentication: None,
        }

        request: {
            pub q2: Foo,

            #[ruma_api(path)]
            pub bar: String,

            #[ruma_api(query)]
            pub baz: OwnedUserId,

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
