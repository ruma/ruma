//! `PUT /_matrix/media/*/upload/{serverName}/{mediaId}`
//!
//! Upload media to an MXC URI that was created with create_mxc_uri.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixmediav3uploadservernamemediaid

    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, IdParseError, MxcUri, OwnedServerName,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("fi.mau.msc2246") => "/_matrix/media/unstable/fi.mau.msc2246/upload/:server_name/:media_id",
            1.7 => "/_matrix/media/v3/upload/:server_name/:media_id",
        }
    };

    /// Request type for the `create_content_async` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

        /// The file contents to upload.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,

        /// The content type of the file being uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<String>,

        /// The name of the file being uploaded.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filename: Option<String>,
        // TODO: How does this and msc2448 (blurhash) interact?
    }

    /// Response type for the `create_content_async` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given file contents.
        pub fn new(media_id: String, server_name: OwnedServerName, file: Vec<u8>) -> Self {
            Self { media_id, server_name, file, content_type: None, filename: None }
        }

        /// Creates a new `Request` with the given url and file contents.
        pub fn from_url(url: &MxcUri, file: Vec<u8>) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;
            Ok(Self::new(media_id.to_owned(), server_name.to_owned(), file))
        }
    }
}
