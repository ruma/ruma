use ruma_api::ruma_api;
use ruma_events::{tag::TagEvent, AnyRoomEvent, EventJson};

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

        // You can use serde attributes on any kind of field
        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_flag: Option<bool>,

        // Use `EventJson` instead of the actual event to allow additional fields to be sent...
        pub event: EventJson<TagEvent>,

        // ... and to allow unknown events when the endpoint deals with event collections.
        pub list_of_events: Vec<EventJson<AnyRoomEvent>>,
    }
}

fn main() {}
