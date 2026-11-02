//! `GET /_matrix/media/*/config`
//!
//! Gets the config for the media repository.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.19/client-server-api/#get_matrixmediav3config

    use js_int::UInt;
    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/media/r0/config",
            1.1 => "/_matrix/media/v3/config",
            1.11 => deprecated,
        }
    }

    /// Request type for the `get_media_config` endpoint.
    #[request]
    #[derive(Default)]
    #[deprecated = "\
        Since Matrix 1.11, clients should use `authenticated_media::get_media_config::v1::Request` \
        instead if the homeserver supports it.\
    "]
    pub struct Request {}

    /// Response type for the `get_media_config` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Maximum size of upload in bytes.
        ///
        /// If this is `None`, the server does not advertise an upload size limit, but this
        /// doesn't mean that there is no limit.
        #[serde(rename = "m.upload.size", skip_serializing_if = "Option::is_none")]
        pub upload_size: Option<UInt>,
    }

    #[allow(deprecated)]
    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given maximum upload size.
        ///
        /// If `upload_size` is `None`, `m.upload.size` is omitted from the serialized response.
        pub fn new(upload_size: Option<UInt>) -> Self {
            Self { upload_size }
        }
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use js_int::uint;
    use ruma_common::api::IncomingResponseExt as _;
    use serde_json::json;

    use super::v3::Response;

    fn deserialize(body: serde_json::Value) -> Response {
        let body = body.to_string();
        Response::try_from_http_response(http::Response::new(body.as_bytes())).unwrap()
    }

    #[test]
    fn deserialize_response_with_upload_size() {
        let response = deserialize(json!({ "m.upload.size": 50_000_000 }));
        assert_eq!(response.upload_size, Some(uint!(50_000_000)));
    }

    #[test]
    fn deserialize_response_without_upload_size() {
        assert_eq!(deserialize(json!({})).upload_size, None);
        assert_eq!(deserialize(json!({ "m.upload.size": null })).upload_size, None);
    }
}

#[cfg(all(test, feature = "server"))]
mod server_tests {
    use js_int::uint;
    use ruma_common::api::OutgoingResponseExt as _;
    use serde_json::{Value as JsonValue, json};

    use super::v3::Response;

    fn serialize(response: Response) -> JsonValue {
        let response: http::Response<Vec<u8>> = response.try_into_http_response().unwrap();
        serde_json::from_slice(response.body()).unwrap()
    }

    #[test]
    fn serialize_response_with_upload_size() {
        assert_eq!(
            serialize(Response::new(Some(uint!(50_000_000)))),
            json!({ "m.upload.size": 50_000_000 })
        );
    }

    #[test]
    fn serialize_response_without_upload_size() {
        assert_eq!(serialize(Response::new(None)), json!({}));
    }
}
