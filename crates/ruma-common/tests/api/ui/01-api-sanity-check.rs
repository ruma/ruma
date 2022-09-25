use ruma_common::{
    api::ruma_api,
    events::{tag::TagEvent, AnyTimelineEvent},
    serde::Raw,
};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST, // An `http::Method` constant. No imports required.
        name: "some_endpoint",
        unstable_path: "/_matrix/some/msc1234/endpoint/:baz",
        r0_path: "/_matrix/some/r0/endpoint/:baz",
        stable_path: "/_matrix/some/v3/endpoint/:baz",
        rate_limited: false,
        authentication: None,
        added: 1.0,
        deprecated: 1.2,
        removed: 1.3,

        history: {
            unstable => "/_matrix/some/msc1234/endpoint/:baz",

            1.0 => "/_matrix/some/r0/endpoint/:baz",
            1.1 => "/_matrix/some/v3/endpoint/:baz",

            1.2 => deprecated,
            1.3 => removed,
        }
    }

    request: {
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

    response: {
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

fn main() {
    use ruma_common::api::MatrixVersion;

    assert_eq!(METADATA.history.all_unstable_paths(), &["/_matrix/some/msc1234/endpoint/:baz"],);
    assert_eq!(
        METADATA.history.all_versioned_stable_paths(),
        &[
            (MatrixVersion::V1_0, "/_matrix/some/r0/endpoint/:baz"),
            (MatrixVersion::V1_1, "/_matrix/some/v3/endpoint/:baz")
        ]
    );

    assert_eq!(METADATA.history.added_version(), Some(MatrixVersion::V1_0));
    assert_eq!(METADATA.history.deprecated, Some(MatrixVersion::V1_2));
    assert_eq!(METADATA.history.removed, Some(MatrixVersion::V1_3));
}
