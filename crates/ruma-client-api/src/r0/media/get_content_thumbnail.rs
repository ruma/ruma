//! [GET /_matrix/media/r0/thumbnail/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-media-r0-thumbnail-servername-mediaid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::{Error, MxcUri, ServerName};
use ruma_serde::StringEnum;

ruma_api! {
    metadata: {
        description: "Get a thumbnail of content from the media store.",
        method: GET,
        name: "get_content_thumbnail",
        path: "/_matrix/media/r0/thumbnail/:server_name/:media_id",
        rate_limited: true,
        authentication: None,
    }

    request: {
        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: &'a str,

        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: &'a ServerName,

        /// The desired resizing method.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub method: Option<Method>,

        /// The *desired* width of the thumbnail. The actual thumbnail may not match the size
        /// specified.
        #[ruma_api(query)]
        pub width: UInt,

        /// The *desired* height of the thumbnail. The actual thumbnail may not match the size
        /// specified.
        #[ruma_api(query)]
        pub height: UInt,

        /// Whether to fetch media deemed remote.
        ///
        /// Used to prevent routing loops. Defaults to `true`.
        #[ruma_api(query)]
        #[serde(default = "ruma_serde::default_true", skip_serializing_if = "ruma_serde::is_true")]
        pub allow_remote: bool,
    }

    response: {
        /// A thumbnail of the requested content.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,

        /// The content type of the thumbnail.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<String>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given media ID, server name, desired thumbnail width and
    /// desired thumbnail height.
    pub fn new(media_id: &'a str, server_name: &'a ServerName, width: UInt, height: UInt) -> Self {
        Self { media_id, server_name, method: None, width, height, allow_remote: true }
    }

    /// Creates a new `Request` with the given url, desired thumbnail width and
    /// desired thumbnail height.
    pub fn from_url(url: &'a MxcUri, width: UInt, height: UInt) -> Result<Self, Error> {
        if let Some((server_name, media_id)) = url.parts() {
            Ok(Self { media_id, server_name, method: None, width, height, allow_remote: true })
        } else {
            Err(Error::InvalidMxcUri)
        }
    }
}

impl Response {
    /// Creates a new `Response` with the given thumbnail.
    pub fn new(file: Vec<u8>) -> Self {
        Self { file, content_type: None }
    }
}

/// The desired resizing method.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Method {
    /// Crop the original to produce the requested image dimensions.
    Crop,

    /// Maintain the original aspect ratio of the source image.
    Scale,

    #[doc(hidden)]
    _Custom(String),
}

impl Method {
    /// Creates a string slice from this `Method`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
