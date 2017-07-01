//! Endpoints for the media repository.

/// [GET /_matrix/media/r0/download/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-media-r0-download-servername-mediaid)
pub mod get_content {
    use hyper::header::{ContentDisposition, ContentType};
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Retrieve content from the media store.",
            method: Method::Get,
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
        }

        response {
            /// The content that was previously uploaded.
            #[ruma_api(body)]
            pub file: Vec<u8>,
            /// The content type of the file that was previously uploaded.
            #[ruma_api(header)]
            pub content_type: ContentType,
            /// The name of the file that was previously uploaded, if set.
            #[ruma_api(header)]
            pub content_disposition: ContentDisposition,
        }
    }
}

/// [POST /_matrix/media/r0/upload](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-media-r0-upload)
pub mod create_content {
    use hyper::header::ContentType;
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Upload content to the media store.",
            method: Method::Post,
            name: "create_media_content",
            path: "/_matrix/media/r0/upload",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// The content type of the file being uploaded.
            #[ruma_api(header)]
            pub content_type: ContentType,
        }

        response {
            /// The MXC URI for the uploaded content.
            pub content_uri: String,
        }
    }
}

/// [GET /_matrix/media/r0/thumbnail/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-media-r0-thumbnail-servername-mediaid)
pub mod get_content_thumbnail {
    use ruma_api_macros::ruma_api;

    /// The desired resizing method.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum Method {
        /// Crop the original to produce the requested image dimensions.
        #[serde(rename = "crop")]
        Crop,
        /// Maintain the original aspect ratio of the source image.
        #[serde(rename = "scale")]
        Scale,
    }

    ruma_api! {
        metadata {
            description: "Get a thumbnail of content from the media store.",
            method: Method::Get,
            name: "get_content_thumbnail",
            path: "/_matrix/media/r0/thumbnail/:server_name/:media_id",
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
            /// The *desired* height of the thumbnail. The actual thumbnail may not match the size
            /// specified.
            #[ruma_api(query)]
            pub height: Option<u64>,
            /// The desired resizing method.
            #[ruma_api(query)]
            pub method: Option<Method>,
            /// The *desired* width of the thumbnail. The actual thumbnail may not match the size
            /// specified.
            #[ruma_api(query)]
            pub width: Option<u64>,
        }

        response {
            /// A thumbnail of the requested content.
            #[ruma_api(body)]
            pub file: Vec<u8>,
        }
    }
}
