//! [GET /_matrix/media/r0/config](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-media-r0-config)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Gets the config for the media repository.",
        method: GET,
        path: "/_matrix/media/r0/config",
        name: "get_media_config",
        rate_limited: true,
        authentication: AccessToken,
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
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given maximum upload size.
    pub fn new(upload_size: UInt) -> Self {
        Self { upload_size }
    }
}
