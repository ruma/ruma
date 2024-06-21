//! `GET /_matrix/media/*/config`
//!
//! Gets the config for the media repository.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3config

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/media/r0/config",
            1.1 => "/_matrix/media/v3/config",
            1.11 => deprecated,
        }
    };

    /// Request type for the `get_media_config` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    #[deprecated = "\
        Since Matrix 1.11, clients should use `authenticated_media::get_media_config::v1::Request` \
        instead if the homeserver supports it.\
    "]
    pub struct Request {}

    /// Response type for the `get_media_config` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Maximum size of upload in bytes.
        #[serde(rename = "m.upload.size")]
        pub upload_size: UInt,
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
        pub fn new(upload_size: UInt) -> Self {
            Self { upload_size }
        }
    }
}
