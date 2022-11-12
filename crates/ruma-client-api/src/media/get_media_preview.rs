//! `GET /_matrix/media/*/preview_url`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixmediav3preview_url

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, MilliSecondsSinceUnixEpoch,
    };
    use serde::Serialize;
    use serde_json::value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue};

    const METADATA: Metadata = metadata! {
        description: "Get a preview for a URL.",
        method: GET,
        name: "get_media_preview",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/media/r0/preview_url",
            1.1 => "/_matrix/media/v3/preview_url",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// URL to get a preview of.
        #[ruma_api(query)]
        pub url: &'a str,

        /// Preferred point in time (in milliseconds) to return a preview for.
        #[ruma_api(query)]
        pub ts: MilliSecondsSinceUnixEpoch,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// OpenGraph-like data for the URL.
        ///
        /// Differences from OpenGraph: the image size in bytes is added to the `matrix:image:size`
        /// field, and `og:image` returns the MXC URI to the image, if any.
        #[ruma_api(body)]
        pub data: Option<Box<RawJsonValue>>,
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
        use assert_matches::assert_matches;
        use serde_json::{
            from_value as from_json_value, json,
            value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue},
        };

        // Since BTreeMap<String, Box<RawJsonValue>> deserialization doesn't seem to
        // work, test that Option<RawJsonValue> works
        #[test]
        fn raw_json_deserialize() {
            type OptRawJson = Option<Box<RawJsonValue>>;

            assert_matches!(from_json_value::<OptRawJson>(json!(null)).unwrap(), None);
            from_json_value::<OptRawJson>(json!("test")).unwrap().unwrap();
            from_json_value::<OptRawJson>(json!({ "a": "b" })).unwrap().unwrap();
        }

        // For completeness sake, make sure serialization works too
        #[test]
        fn raw_json_serialize() {
            to_raw_json_value(&json!(null)).unwrap();
            to_raw_json_value(&json!("string")).unwrap();
            to_raw_json_value(&json!({})).unwrap();
            to_raw_json_value(&json!({ "a": "b" })).unwrap();
        }
    }
}
