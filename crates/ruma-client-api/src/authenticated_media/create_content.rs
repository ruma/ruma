//! `POST /_matrix/client/*/media/upload`
//!
//! Upload content to the media store.

pub mod unstable {
    //! `/unstable/org.matrix.msc3911/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3911

    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3911/media/upload",
        }
    };

    /// Request type for the `create_content` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The name of the file being uploaded.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filename: Option<String>,

        /// The content type of the file being uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<String>,

        /// Should the server return a blurhash or not.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[ruma_api(query)]
        #[cfg(feature = "unstable-msc2448")]
        #[serde(
            default,
            skip_serializing_if = "ruma_common::serde::is_default",
            rename = "xyz.amorgan.generate_blurhash"
        )]
        pub generate_blurhash: bool,

        /// The file contents to upload.
        #[ruma_api(raw_body)]
        pub file: Vec<u8>,
    }

    /// Response type for the `create_content` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The MXC URI for the uploaded content.
        pub content_uri: OwnedMxcUri,

        /// The [BlurHash](https://blurha.sh) for the uploaded content.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(
            rename = "xyz.amorgan.blurhash",
            alias = "blurhash",
            skip_serializing_if = "Option::is_none"
        )]
        pub blurhash: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given file contents.
        pub fn new(file: Vec<u8>) -> Self {
            Self {
                file,
                filename: None,
                content_type: None,
                #[cfg(feature = "unstable-msc2448")]
                generate_blurhash: false,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given MXC URI.
        pub fn new(content_uri: OwnedMxcUri) -> Self {
            Self {
                content_uri,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
            }
        }
    }
}
