#![allow(unexpected_cfgs)]

use http::header::CONTENT_TYPE;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
    serde::Raw,
};

const METADATA: Metadata = metadata! {
    method: POST, // An `http::Method` constant. No imports required.
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/some/msc1234/endpoint/:baz",
        unstable("org.bar.msc1234") => "/_matrix/unstable/org.bar.msc1234/endpoint/:baz",
        stable("org.bar.msc1234.stable") => "/_matrix/some/r0/endpint/:baz",
        1.0 | stable("org.bar.msc1234.stable.v2") => "/_matrix/some/r0/endpoint/:baz",
        1.1 => "/_matrix/some/v3/endpoint/:baz",
        1.2 => deprecated,
        1.3 => removed,
    }
};

/// Request type for the `some_endpoint` endpoint.
#[request]
pub struct Request {
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

fn main() {
    use ruma_common::api::{MatrixVersion, StablePathSelector};

    assert_eq!(
        METADATA.history.unstable_paths().collect::<Vec<_>>(),
        &[
            (None, "/_matrix/some/msc1234/endpoint/:baz"),
            (Some("org.bar.msc1234"), "/_matrix/unstable/org.bar.msc1234/endpoint/:baz")
        ],
    );
    assert_eq!(
        METADATA.history.stable_paths().collect::<Vec<_>>(),
        &[
            (
                StablePathSelector::Feature("org.bar.msc1234.stable"),
                "/_matrix/some/r0/endpint/:baz"
            ),
            (
                StablePathSelector::FeatureAndVersion {
                    feature: "org.bar.msc1234.stable.v2",
                    version: MatrixVersion::V1_0
                },
                "/_matrix/some/r0/endpoint/:baz"
            ),
            (StablePathSelector::Version(MatrixVersion::V1_1), "/_matrix/some/v3/endpoint/:baz")
        ],
    );
    assert_eq!(
        METADATA.history.all_paths().collect::<Vec<_>>(),
        &[
            "/_matrix/some/msc1234/endpoint/:baz",
            "/_matrix/unstable/org.bar.msc1234/endpoint/:baz",
            "/_matrix/some/r0/endpint/:baz",
            "/_matrix/some/r0/endpoint/:baz",
            "/_matrix/some/v3/endpoint/:baz"
        ],
    );

    assert_eq!(METADATA.history.added_in(), Some(MatrixVersion::V1_0));
    assert_eq!(METADATA.history.deprecated_in(), Some(MatrixVersion::V1_2));
    assert_eq!(METADATA.history.removed_in(), Some(MatrixVersion::V1_3));
}
