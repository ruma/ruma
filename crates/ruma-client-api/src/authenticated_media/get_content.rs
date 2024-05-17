//! `GET /_matrix/client/*/media/download/{serverName}/{mediaId}`
//!
//! Retrieve content from the media store.

pub mod unstable {
    //! `/unstable/org.matrix.msc3916/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916

    use std::time::Duration;

    use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, IdParseError, MxcUri, OwnedServerName,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3916/media/download/:server_name/:media_id",
        }
    };

    /// Request type for the `get_media_content` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

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
    }

    /// Response type for the `get_media_content` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The content that was previously uploaded.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,

        /// The content type of the file that was previously uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<String>,

        /// The value of the `Content-Disposition` HTTP header, possibly containing the name of the
        /// file that was previously uploaded.
        ///
        /// See [MDN] for the syntax.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition#Syntax
        #[ruma_api(header = CONTENT_DISPOSITION)]
        pub content_disposition: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID and server name.
        pub fn new(media_id: String, server_name: OwnedServerName) -> Self {
            Self { media_id, server_name, timeout_ms: crate::media::default_download_timeout() }
        }

        /// Creates a new `Request` with the given URI.
        pub fn from_uri(uri: &MxcUri) -> Result<Self, IdParseError> {
            let (server_name, media_id) = uri.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned()))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given file contents.
        pub fn new(file: Vec<u8>) -> Self {
            Self { file, content_type: None, content_disposition: None }
        }
    }
}
