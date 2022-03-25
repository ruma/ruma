//! `GET /_matrix/media/*/download/{serverName}/{mediaId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixmediav3downloadservernamemediaid

    use ruma_common::{api::ruma_api, IdParseError, MxcUri, ServerName};

    ruma_api! {
        metadata: {
            description: "Retrieve content from the media store.",
            method: GET,
            name: "get_media_content",
            r0_path: "/_matrix/media/r0/download/:server_name/:media_id",
            stable_path: "/_matrix/media/v3/download/:server_name/:media_id",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The media ID from the mxc:// URI (the path component).
            #[ruma_api(path)]
            pub media_id: &'a str,

            /// The server name from the mxc:// URI (the authoritory component).
            #[ruma_api(path)]
            pub server_name: &'a ServerName,

            /// Whether to fetch media deemed remote.
            ///
            /// Used to prevent routing loops. Defaults to `true`.
            #[ruma_api(query)]
            #[serde(default = "ruma_common::serde::default_true", skip_serializing_if = "ruma_common::serde::is_true")]
            pub allow_remote: bool,
        }

        response: {
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

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given media ID and server name.
        pub fn new(media_id: &'a str, server_name: &'a ServerName) -> Self {
            Self { media_id, server_name, allow_remote: true }
        }

        /// Creates a new `Request` with the given url.
        pub fn from_url(url: &'a MxcUri) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self { media_id, server_name, allow_remote: true })
        }
    }

    impl Response {
        /// Creates a new `Response` with the given file contents.
        pub fn new(file: Vec<u8>) -> Self {
            Self { file, content_type: None, content_disposition: None }
        }
    }
}
