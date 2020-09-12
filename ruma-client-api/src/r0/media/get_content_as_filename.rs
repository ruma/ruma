//! [GET /_matrix/media/r0/download/{serverName}/{mediaId}/{fileName}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-media-r0-download-servername-mediaid-filename)

use ruma_api::ruma_api;
use ruma_identifiers::ServerName;

ruma_api! {
    metadata: {
        description: "Retrieve content from the media store, specifying a filename to return.",
        method: GET,
        name: "get_media_content_as_filename",
        path: "/_matrix/media/r0/download/:server_name/:media_id/:filename",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: &'a str,

        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: &'a ServerName,

        /// The filename to return in the `Content-Disposition` header.
        #[ruma_api(path)]
        pub filename: &'a str,

        /// Whether to fetch media deemed remote.
        ///
        /// Used to prevent routing loops. Defaults to `true`.
        #[ruma_api(query)]
        #[serde(default = "ruma_serde::default_true", skip_serializing_if = "ruma_serde::is_true")]
        pub allow_remote: bool,
    }

    response: {
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

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given media ID, server name and filename.
    pub fn new(media_id: &'a str, server_name: &'a ServerName, filename: &'a str) -> Self {
        Self { media_id, server_name, filename, allow_remote: true }
    }
}

impl Response {
    /// Creates a new `Response` with the given file, content type and filename.
    pub fn new(file: Vec<u8>, content_type: String, content_disposition: String) -> Self {
        Self { file, content_type, content_disposition }
    }
}
