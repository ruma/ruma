#![allow(clippy::exhaustive_structs)]

#[cfg(feature = "events")]
pub mod some_endpoint {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        events::{tag::TagEvent, AnyTimelineEvent},
        metadata,
        serde::Raw,
        OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/endpoint/:user",
        }
    };

    #[request]
    pub struct Request {
        // With no attribute on the field, it will be put into the body of the request.
        pub a_field: String,

        // This value will be put into the "Content-Type" HTTP header.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,

        // This value will be put into the query string of the request's URL.
        #[ruma_api(query)]
        pub bar: String,

        // This value will be inserted into the request's URL in place of the
        // ":user" path component.
        #[ruma_api(path)]
        pub user: OwnedUserId,
    }

    #[response]
    pub struct Response {
        // This value will be extracted from the "Content-Type" HTTP header.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,

        // With no attribute on the field, it will be extracted from the body of the response.
        pub value: String,

        // You can use serde attributes on any kind of field
        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_flag: Option<bool>,

        // Use `Raw` instead of the actual event to allow additional fields to be sent...
        pub event: Raw<TagEvent>,

        // ... and to allow unknown events when the endpoint deals with event collections.
        pub list_of_events: Vec<Raw<AnyTimelineEvent>>,
    }
}

pub mod newtype_body_endpoint {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct MyCustomType {
        pub a_field: String,
    }

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: PUT,
        name: "newtype_body_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/newtype/body/endpoint",
        }
    };

    #[request]
    pub struct Request {
        #[ruma_api(body)]
        pub list_of_custom_things: Vec<MyCustomType>,
    }

    #[response]
    pub struct Response {
        #[ruma_api(body)]
        pub my_custom_thing: MyCustomType,
    }
}

pub mod raw_body_endpoint {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct MyCustomType {
        pub a_field: String,
    }

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: PUT,
        name: "newtype_body_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/newtype/body/endpoint",
        }
    };

    #[request]
    pub struct Request<'a> {
        #[ruma_api(raw_body)]
        pub file: &'a [u8],
    }

    #[response]
    pub struct Response {
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
    }
}

pub mod query_map_endpoint {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Does something.",
        method: GET,
        name: "newtype_body_endpoint",
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/query/map/endpoint",
        }
    };

    #[request]
    pub struct Request {
        #[ruma_api(query_map)]
        pub fields: Vec<(String, String)>,
    }

    #[response]
    pub struct Response {}
}
