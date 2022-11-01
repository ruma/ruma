//! `GET /_matrix/media/*/config`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixmediav3config

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Gets the config for the media repository.",
        method: GET,
        name: "get_media_config",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/media/r0/config",
            1.1 => "/_matrix/media/v3/config",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

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
