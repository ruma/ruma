//! `GET /_matrix/media/*/config`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixmediav3config

    use js_int::UInt;
    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Gets the config for the media repository.",
            method: GET,
            r0_path: "/_matrix/media/r0/config",
            stable_path: "/_matrix/media/v3/config",
            name: "get_media_config",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// Maximum size of upload in bytes.
            #[serde(rename = "m.upload.size")]
            pub upload_size: UInt,
        }

        error: crate::Error
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
