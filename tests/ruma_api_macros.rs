pub mod some_endpoint {
    use ruma_api::ruma_api;

    ruma_api! {
        metadata {
            description: "Does something.",
            method: POST, // An `http::Method` constant. No imports required.
            name: "some_endpoint",
            path: "/_matrix/some/endpoint/:baz",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            // With no attribute on the field, it will be put into the body of the request.
            pub foo: String,

            // This value will be put into the "Content-Type" HTTP header.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: String,

            // This value will be put into the query string of the request's URL.
            #[ruma_api(query)]
            pub bar: String,

            // This value will be inserted into the request's URL in place of the
            // ":baz" path component.
            #[ruma_api(path)]
            pub baz: String,
        }

        response {
            // This value will be extracted from the "Content-Type" HTTP header.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: String,

            // With no attribute on the field, it will be extracted from the body of the response.
            pub value: String,
        }
    }
}

pub mod newtype_body_endpoint {
    use ruma_api_macros::ruma_api;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct MyCustomType {
        pub foo: String,
    }

    ruma_api! {
        metadata {
            description: "Does something.",
            method: PUT,
            name: "newtype_body_endpoint",
            path: "/_matrix/some/newtype/body/endpoint",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            #[ruma_api(body)]
            pub file: Vec<u8>,
        }

        response {
            #[ruma_api(body)]
            pub my_custom_type: MyCustomType,
        }
    }
}
