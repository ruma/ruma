#![allow(clippy::exhaustive_structs)]

pub mod some_endpoint {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: POST, // An `http::Method` constant. No imports required.
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/endpoint/{user}",
        }
    };

    /// Request type for the `some_endpoint` endpoint.
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

    /// Response type for the `some_endpoint` endpoint.
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
        pub event: Raw<Event>,

        // ... and to allow unknown events when the endpoint deals with event collections.
        pub list_of_events: Vec<Raw<Event>>,
    }

    // Dummy type to avoid circular dev-dependency that rust-analyzer doesn't like
    pub struct Event {}
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
        method: PUT,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/newtype/body/endpoint",
        }
    };

    /// Request type for the `newtype_body_endpoint` endpoint.
    #[request]
    pub struct Request {
        #[ruma_api(body)]
        pub list_of_custom_things: Vec<MyCustomType>,
    }

    /// Response type for the `newtype_body_endpoint` endpoint.
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
        method: PUT,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/newtype/body/endpoint",
        }
    };

    /// Request type for the `newtype_body_endpoint` endpoint.
    #[request]
    pub struct Request {
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
    }

    /// Response type for the `newtype_body_endpoint` endpoint.
    #[response]
    pub struct Response {
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
    }
}

pub mod query_all_enum_endpoint {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    #[serde(untagged)]
    pub enum MyCustomQueryEnum {
        VariantA { field_a: String },
        VariantB { field_b: String },
    }

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/query/map/endpoint",
        }
    };

    /// Request type for the `query_all_enum_endpoint` endpoint.
    #[request]
    pub struct Request {
        #[ruma_api(query_all)]
        pub query: MyCustomQueryEnum,
    }

    /// Response type for the `query_all_enum_endpoint` endpoint.
    #[response]
    pub struct Response {}
}

pub mod query_all_vec_endpoint {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/some/query/map/endpoint",
        }
    };

    /// Request type for the `query_all_vec_endpoint` endpoint.
    #[request]
    pub struct Request {
        #[ruma_api(query_all)]
        pub fields: Vec<(String, String)>,
    }

    /// Response type for the `query_all_vec_endpoint` endpoint.
    #[response]
    pub struct Response {}
}
