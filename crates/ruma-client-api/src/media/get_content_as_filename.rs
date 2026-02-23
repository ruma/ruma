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
        IdParseError, MxcUri, ServerName,
        api::{auth_scheme::NoAuthentication, request, response},
        http_headers::ContentDisposition,
        metadata,
    };

    use crate::http_headers::CROSS_ORIGIN_RESOURCE_POLICY;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/media/r0/download/{server_name}/{media_id}/{filename}",
            1.1 => "/_matrix/media/v3/download/{server_name}/{media_id}/{filename}",
            1.11 => deprecated,
        }
    }

    /// Request type for the `get_media_content_as_filename` endpoint.
    #[request(error = crate::Error)]
    #[deprecated = "\
        Since Matrix 1.11, clients should use `authenticated_media::get_content_as_filename::v1::Request` \
        instead if the homeserver supports it.\
    "]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: ServerName,

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
            default = "ruma_common::media::default_download_timeout",
            skip_serializing_if = "ruma_common::media::is_default_download_timeout"
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
        #[ruma_api(header = CONTENT_DISPOSITION)]
        pub content_disposition: Option<ContentDisposition>,

        /// The value of the `Cross-Origin-Resource-Policy` HTTP header.
        ///
        /// See [MDN] for the syntax.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy#syntax
        #[ruma_api(header = CROSS_ORIGIN_RESOURCE_POLICY)]
        pub cross_origin_resource_policy: Option<String>,
    }

    #[allow(deprecated)]
    impl Request {
        /// Creates a new `Request` with the given media ID, server name and filename.
        pub fn new(media_id: String, server_name: ServerName, filename: String) -> Self {
            Self {
                media_id,
                server_name,
                filename,
                allow_remote: true,
                timeout_ms: ruma_common::media::default_download_timeout(),
                allow_redirect: false,
            }
        }

        /// Creates a new `Request` with the given url and filename.
        pub fn from_url(url: &MxcUri, filename: String) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name, filename))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given file.
        ///
        /// The Cross-Origin Resource Policy defaults to `cross-origin`.
        pub fn new(
            file: Vec<u8>,
            content_type: String,
            content_disposition: ContentDisposition,
        ) -> Self {
            Self {
                file,
                content_type: Some(content_type),
                content_disposition: Some(content_disposition),
                cross_origin_resource_policy: Some("cross-origin".to_owned()),
            }
        }
    }
}
