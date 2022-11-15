//! `GET /_matrix/media/*/download/{serverName}/{mediaId}`
//!
//! Retrieve content from the media store.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixmediav3downloadservernamemediaid

    use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    #[cfg(feature = "unstable-msc2246")]
    use js_int::UInt;
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
            1.0 => "/_matrix/media/r0/download/:server_name/:media_id",
            1.1 => "/_matrix/media/v3/download/:server_name/:media_id",
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

        /// Whether to fetch media deemed remote.
        ///
        /// Used to prevent routing loops. Defaults to `true`.
        #[ruma_api(query)]
        #[serde(
            default = "ruma_common::serde::default_true",
            skip_serializing_if = "ruma_common::serde::is_true"
        )]
        pub allow_remote: bool,

        /// How long to wait for the media to be uploaded
        ///
        /// This uses the unstable prefix in
        /// [MSC2246](https://github.com/matrix-org/matrix-spec-proposals/pull/2246)
        #[ruma_api(query)]
        #[cfg(feature = "unstable-msc2246")]
        #[serde(
            default,
            skip_serializing_if = "ruma_common::serde::is_default",
            rename = "fi.mau.msc2246.max_stall_ms"
        )]
        pub max_stall_ms: Option<UInt>,
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

        /// The value of the `Cross-Origin-Resource-Policy` HTTP header.
        ///
        /// See [MDN] for the syntax.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy#syntax
        #[ruma_api(header = CROSS_ORIGIN_RESOURCE_POLICY)]
        pub cross_origin_resource_policy: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID and server name.
        pub fn new(media_id: String, server_name: OwnedServerName) -> Self {
            Self {
                media_id,
                server_name,
                allow_remote: true,
                #[cfg(feature = "unstable-msc2246")]
                max_stall_ms: None,
            }
        }

        /// Creates a new `Request` with the given url.
        pub fn from_url(url: &MxcUri) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned()))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given file contents.
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
