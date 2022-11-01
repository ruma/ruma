// This tests that the "body" fields are moved after all other fields because they
// consume the request/response.

mod newtype_body {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/foo/:bar/",
        }
    };

    #[request]
    pub struct Request {
        #[ruma_api(body)]
        pub q2: Foo,

        #[ruma_api(path)]
        pub bar: String,

        #[ruma_api(query)]
        pub baz: OwnedUserId,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }

    #[response]
    pub struct Response {
        #[ruma_api(body)]
        pub q2: Foo,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }
}

mod raw_body {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/foo/:bar/",
        }
    };

    #[request]
    pub struct Request {
        #[ruma_api(raw_body)]
        pub q2: Vec<u8>,

        #[ruma_api(path)]
        pub bar: String,

        #[ruma_api(query)]
        pub baz: OwnedUserId,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }

    #[response]
    pub struct Response {
        #[ruma_api(raw_body)]
        pub q2: Vec<u8>,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }
}

mod plain {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Foo;

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/foo/:bar/",
        }
    };

    #[request]
    pub struct Request {
        pub q2: Foo,

        #[ruma_api(path)]
        pub bar: String,

        #[ruma_api(query)]
        pub baz: OwnedUserId,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }

    #[response]
    pub struct Response {
        pub q2: Vec<u8>,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
    }
}

fn main() {}
