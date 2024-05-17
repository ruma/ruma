//! `GET /_matrix/client/*/media/config`
//!
//! Gets the config for the media repository.

pub mod unstable {
    //! `/unstable/org.matrix.msc3916/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916

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
            unstable => "/_matrix/client/unstable/org.matrix.msc3916/media/config",
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
