//! `GET /_matrix/client/*/media/config`
//!
//! Gets the config for the media repository.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1mediaconfig

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
            unstable("org.matrix.msc3916") => "/_matrix/client/unstable/org.matrix.msc3916/media/config",
            1.11 | stable("org.matrix.msc3916.stable") => "/_matrix/client/v1/media/config",
        }
    };

    /// Request type for the `get_media_config` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_media_config` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Maximum size of upload in bytes.
        #[serde(rename = "m.upload.size")]
        pub upload_size: UInt,
    }

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
