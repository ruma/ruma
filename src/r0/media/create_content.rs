//! [POST /_matrix/media/r0/upload](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-media-r0-upload)

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Upload content to the media store.",
        method: POST,
        name: "create_media_content",
        path: "/_matrix/media/r0/upload",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The name of the file being uploaded.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub filename: Option<String>,
        /// The content type of the file being uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,
        /// The file contents to upload.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
    }

    response {
        /// The MXC URI for the uploaded content.
        pub content_uri: String,
    }

    error: crate::Error
}
