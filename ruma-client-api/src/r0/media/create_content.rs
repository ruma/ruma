//! [POST /_matrix/media/r0/upload](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-media-r0-upload)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Upload content to the media store.",
        method: POST,
        name: "create_media_content",
        path: "/_matrix/media/r0/upload",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The file contents to upload.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,

        /// The name of the file being uploaded.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filename: Option<&'a str>,

        /// The content type of the file being uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<&'a str>,
    }

    response: {
        /// The MXC URI for the uploaded content.
        pub content_uri: String,

        /// The [BlurHash](https://blurha.sh) for the uploaded content.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
        #[cfg(feature = "unstable-pre-spec")]
        #[serde(rename = "xyz.amorgan.blurhash")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<String>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given file contents.
    pub fn new(file: Vec<u8>) -> Self {
        Self { file, filename: None, content_type: None }
    }
}

impl Response {
    /// Creates a new `Response` with the given MXC URI.
    pub fn new(content_uri: String) -> Self {
        Self {
            content_uri,
            #[cfg(feature = "unstable-pre-spec")]
            blurhash: None,
        }
    }
}
