//! `GET /_matrix/client/*/media/thumbnail/{serverName}/{mediaId}`
//!
//! Get a thumbnail of content from the media store.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1mediathumbnailservernamemediaid

    use std::time::Duration;

    use http::header::CONTENT_TYPE;
    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, IdParseError, MxcUri, OwnedServerName,
    };

    use crate::media::get_content_thumbnail::v3::Method;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3916/media/thumbnail/:server_name/:media_id",
            1.11 => "/_matrix/client/v1/media/thumbnail/:server_name/:media_id",
        }
    };

    /// Request type for the `get_content_thumbnail` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

        /// The desired resizing method.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub method: Option<Method>,

        /// The *desired* width of the thumbnail.
        ///
        /// The actual thumbnail may not match the size specified.
        #[ruma_api(query)]
        pub width: UInt,

        /// The *desired* height of the thumbnail.
        ///
        /// The actual thumbnail may not match the size specified.
        #[ruma_api(query)]
        pub height: UInt,

        /// The maximum duration that the client is willing to wait to start receiving data, in the
        /// case that the content has not yet been uploaded.
        ///
        /// The default value is 20 seconds.
        #[ruma_api(query)]
        #[serde(
            with = "ruma_common::serde::duration::ms",
            default = "crate::media::default_download_timeout",
            skip_serializing_if = "crate::media::is_default_download_timeout"
        )]
        pub timeout_ms: Duration,

        /// Whether the server should return an animated thumbnail.
        ///
        /// When `Some(true)`, the server should return an animated thumbnail if possible and
        /// supported. When `Some(false)`, the server must not return an animated
        /// thumbnail. When `None`, the server should not return an animated thumbnail.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub animated: Option<bool>,
    }

    /// Response type for the `get_content_thumbnail` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A thumbnail of the requested content.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,

        /// The content type of the thumbnail.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID, server name, desired thumbnail width
        /// and desired thumbnail height.
        pub fn new(
            media_id: String,
            server_name: OwnedServerName,
            width: UInt,
            height: UInt,
        ) -> Self {
            Self {
                media_id,
                server_name,
                method: None,
                width,
                height,
                timeout_ms: crate::media::default_download_timeout(),
                animated: None,
            }
        }

        /// Creates a new `Request` with the given URI, desired thumbnail width and
        /// desired thumbnail height.
        pub fn from_uri(uri: &MxcUri, width: UInt, height: UInt) -> Result<Self, IdParseError> {
            let (server_name, media_id) = uri.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned(), width, height))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given thumbnail.
        pub fn new(file: Vec<u8>) -> Self {
            Self { file, content_type: None }
        }
    }
}
