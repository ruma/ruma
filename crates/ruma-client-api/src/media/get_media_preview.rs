//! `GET /_matrix/media/*/preview_url`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixmediav3preview_url

    use ruma_common::{api::ruma_api, MilliSecondsSinceUnixEpoch};
    use serde::Serialize;
    use serde_json::value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue};

    ruma_api! {
        metadata: {
            description: "Get a preview for a URL.",
            name: "get_media_preview",
            method: GET,
            r0_path: "/_matrix/media/r0/preview_url",
            stable_path: "/_matrix/media/v3/preview_url",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// URL to get a preview of.
            #[ruma_api(query)]
            pub url: &'a str,

            /// Preferred point in time (in milliseconds) to return a preview for.
            #[ruma_api(query)]
            pub ts: MilliSecondsSinceUnixEpoch,
        }

        #[derive(Default)]
        response: {
            /// OpenGraph-like data for the URL.
            ///
            /// Differences from OpenGraph: the image size in bytes is added to the `matrix:image:size`
            /// field, and `og:image` returns the MXC URI to the image, if any.
            #[ruma_api(body)]
            pub data: Option<Box<RawJsonValue>>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given url and timestamp.
        pub fn new(url: &'a str, ts: MilliSecondsSinceUnixEpoch) -> Self {
            Self { url, ts }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self { data: None }
        }

        /// Creates a new `Response` with the given OpenGraph data (in a
        /// `serde_json::value::RawValue`).
        pub fn from_raw_value(data: Box<RawJsonValue>) -> Self {
            Self { data: Some(data) }
        }

        /// Creates a new `Response` with the given OpenGraph data (in any kind of serializable
        /// object).
        pub fn from_serialize<T: Serialize>(data: &T) -> serde_json::Result<Self> {
            Ok(Self { data: Some(to_raw_json_value(data)?) })
        }
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
}
