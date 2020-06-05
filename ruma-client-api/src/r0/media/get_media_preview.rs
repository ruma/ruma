//! [GET /_matrix/media/r0/preview_url](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-media-r0-preview-url)

use std::time::SystemTime;

use ruma_api::ruma_api;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata {
        description: "Get a preview for a URL.",
        name: "get_media_preview",
        method: GET,
        path: "/_matrix/media/r0/preview_url",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// URL to get a preview of.
        #[ruma_api(query)]
        pub url: String,

        /// Preferred point in time (in milliseconds) to return a preview for.
        #[ruma_api(query)]
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub ts: SystemTime,
    }

    response {
        /// OpenGraph-like data for the URL.
        ///
        /// Differences from OpenGraph: the image size in bytes is added to the `matrix:image:size`
        /// field, and `og:image` returns the MXC URI to the image, if any.
        #[ruma_api(body)]
        pub data: Option<Box<RawJsonValue>>,
    }

    error: crate::Error
}

#[cfg(test)]
mod tests {
    use serde_json::{
        from_value as from_json_value, json,
        value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue},
    };

    // Since BTreeMap<String, Box<RawJsonValue>> deserialization doesn't seem to
    // work, test that Option<RawJsonValue> works
    #[test]
    fn raw_json_deserialize() {
        type OptRawJson = Option<Box<RawJsonValue>>;

        assert!(from_json_value::<OptRawJson>(json!(null)).unwrap().is_none());
        assert!(from_json_value::<OptRawJson>(json!("test")).unwrap().is_some());
        assert!(from_json_value::<OptRawJson>(json!({ "a": "b" })).unwrap().is_some());
    }

    // For completeness sake, make sure serialization works too
    #[test]
    fn raw_json_serialize() {
        assert!(to_raw_json_value(&json!(null)).is_ok());
        assert!(to_raw_json_value(&json!("string")).is_ok());
        assert!(to_raw_json_value(&json!({})).is_ok());
        assert!(to_raw_json_value(&json!({ "a": "b" })).is_ok());
    }
}
