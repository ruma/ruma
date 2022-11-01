//! `POST /_matrix/media/*/upload`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixmediav3upload

    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri,
    };

    const METADATA: Metadata = metadata! {
        description: "Upload content to the media store.",
        method: POST,
        name: "create_media_content",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/media/r0/upload",
            1.1 => "/_matrix/media/v3/upload",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The file contents to upload.
        #[ruma_api(raw_body)]
        pub file: &'a [u8],

        /// The name of the file being uploaded.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filename: Option<&'a str>,

        /// The content type of the file being uploaded.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: Option<&'a str>,

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
    }

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

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given file contents.
        pub fn new(file: &'a [u8]) -> Self {
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
