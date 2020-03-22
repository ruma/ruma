//! [GET /_matrix/media/r0/download/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.6.0.html#get-matrix-media-r0-download-servername-mediaid)

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Retrieve content from the media store.",
        method: GET,
        name: "get_media_content",
        path: "/_matrix/media/r0/download/:server_name/:media_id",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: String,
        /// Whether to fetch media deemed remote.
        /// Used to prevent routing loops. Defaults to `true`.
        #[ruma_api(query)]
        pub allow_remote: Option<bool>,
    }

    response {
        /// The content that was previously uploaded.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
        /// The content type of the file that was previously uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,
        /// The name of the file that was previously uploaded, if set.
        #[ruma_api(header = CONTENT_DISPOSITION)]
        pub content_disposition: String,
    }

    error: crate::Error
}
