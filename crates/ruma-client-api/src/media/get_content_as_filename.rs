//! `GET /_matrix/media/*/download/{serverName}/{mediaId}/{fileName}`
//!
//! Retrieve content from the media store, specifying a filename to return.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3downloadservernamemediaidfilename

    use std::time::Duration;

    use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, IdParseError, MxcUri, OwnedServerName,
    };

    use crate::http_headers::CROSS_ORIGIN_RESOURCE_POLICY;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/media/r0/download/:server_name/:media_id/:filename",
            1.1 => "/_matrix/media/v3/download/:server_name/:media_id/:filename",
        }
    };

    /// Request type for the `get_media_content_as_filename` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

        /// The filename to return in the `Content-Disposition` header.
        #[ruma_api(path)]
        pub filename: String,

        /// Whether to fetch media deemed remote.
        ///
        /// Used to prevent routing loops. Defaults to `true`.
        #[ruma_api(query)]
        #[serde(
            default = "ruma_common::serde::default_true",
            skip_serializing_if = "ruma_common::serde::is_true"
        )]
        pub allow_remote: bool,

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

        /// Whether the server may return a 307 or 308 redirect response that points at the
        /// relevant media content.
        ///
        /// Unless explicitly set to `true`, the server must return the media content itself.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub allow_redirect: bool,
    }

    /// Response type for the `get_media_content_as_filename` endpoint.
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

        /// The value of the `Cross-Origin-Resource-Policy` HTTP header.
        ///
        /// See [MDN] for the syntax.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy#syntax
        #[ruma_api(header = CROSS_ORIGIN_RESOURCE_POLICY)]
        pub cross_origin_resource_policy: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID, server name and filename.
        pub fn new(media_id: String, server_name: OwnedServerName, filename: String) -> Self {
            Self {
                media_id,
                server_name,
                filename,
                allow_remote: true,
                timeout_ms: crate::media::default_download_timeout(),
                allow_redirect: false,
            }
        }

        /// Creates a new `Request` with the given url and filename.
        pub fn from_url(url: &MxcUri, filename: String) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned(), filename))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given file.
        ///
        /// The Cross-Origin Resource Policy defaults to `cross-origin`.
        pub fn new(file: Vec<u8>) -> Self {
            Self {
                file,
                content_type: None,
                content_disposition: None,
                cross_origin_resource_policy: Some("cross-origin".to_owned()),
            }
        }
    }
}
