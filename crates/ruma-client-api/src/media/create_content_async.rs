//! `POST /_matrix/media/*/upload/{serverName}/{mediaId}`

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/tulir/matrix-doc/blob/asynchronous_uploads/proposals/2246-asynchronous-uploads.md

    use http::header::CONTENT_TYPE;
    use ruma_common::{api::ruma_api, IdParseError, MxcUri, ServerName};

    ruma_api! {
        metadata: {
            description: "Upload media to an MXC URI that was created with create_mxc_uri.",
            method: PUT,
            name: "create_content_async",
            unstable_path: "/_matrix/media/unstable/fi.mau.msc2246/upload/:server_name/:media_id",
            rate_limited: true,
            authentication: AccessToken,
        }

        request: {
            /// The media ID from the mxc:// URI (the path component).
            #[ruma_api(path)]
            pub media_id: &'a str,

            /// The server name from the mxc:// URI (the authoritory component).
            #[ruma_api(path)]
            pub server_name: &'a ServerName,

            /// The file contents to upload.
            #[ruma_api(raw_body)]
            pub file: &'a [u8],

            /// The content type of the file being uploaded.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: Option<&'a str>,

            // TODO: How does this and msc2448 (blurhash) interact?
        }

        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given file contents.
        pub fn new(media_id: &'a str, server_name: &'a ServerName, file: &'a [u8]) -> Self {
            Self { media_id, server_name, file, content_type: None }
        }

        /// Creates a new `Request` with the given url and file contents.
        pub fn from_url(url: &'a MxcUri, file: &'a [u8]) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;
            Ok(Self::new(media_id, server_name, file))
        }
    }
}
