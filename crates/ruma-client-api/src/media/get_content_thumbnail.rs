//! `GET /_matrix/media/*/thumbnail/{serverName}/{mediaId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixmediav3thumbnailservernamemediaid

    use http::header::CONTENT_TYPE;
    use js_int::UInt;
    use ruma_common::{api::ruma_api, serde::StringEnum, IdParseError, MxcUri, ServerName};

    use crate::{http_headers::CROSS_ORIGIN_RESOURCE_POLICY, PrivOwnedStr};

    ruma_api! {
        metadata: {
            description: "Get a thumbnail of content from the media store.",
            method: GET,
            name: "get_content_thumbnail",
            r0_path: "/_matrix/media/r0/thumbnail/:server_name/:media_id",
            stable_path: "/_matrix/media/v3/thumbnail/:server_name/:media_id",
            rate_limited: true,
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

            /// Whether to fetch media deemed remote.
            ///
            /// Used to prevent routing loops. Defaults to `true`.
            #[ruma_api(query)]
            #[serde(default = "ruma_common::serde::default_true", skip_serializing_if = "ruma_common::serde::is_true")]
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
                rename = "fi.mau.msc2246.max_stall_ms",
            )]
            pub max_stall_ms: Option<UInt>,
        }

        response: {
            /// A thumbnail of the requested content.
            #[ruma_api(raw_body)]
            pub file: Vec<u8>,

            /// The content type of the thumbnail.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: Option<String>,

            /// The value of the `Cross-Origin-Resource-Policy` HTTP header.
            ///
            /// See [MDN] for the syntax.
            ///
            /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy#syntax
            #[ruma_api(header = CROSS_ORIGIN_RESOURCE_POLICY)]
            pub cross_origin_resource_policy: Option<String>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given media ID, server name, desired thumbnail width
        /// and desired thumbnail height.
        pub fn new(
            media_id: &'a str,
            server_name: &'a ServerName,
            width: UInt,
            height: UInt,
        ) -> Self {
            Self {
                media_id,
                server_name,
                method: None,
                width,
                height,
                allow_remote: true,
                #[cfg(feature = "unstable-msc2246")]
                max_stall_ms: None,
            }
        }

        /// Creates a new `Request` with the given url, desired thumbnail width and
        /// desired thumbnail height.
        pub fn from_url(url: &'a MxcUri, width: UInt, height: UInt) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id, server_name, width, height))
        }
    }

    impl Response {
        /// Creates a new `Response` with the given thumbnail.
        ///
        /// The Cross-Origin Resource Policy defaults to `cross-origin`.
        pub fn new(file: Vec<u8>) -> Self {
            Self {
                file,
                content_type: None,
                cross_origin_resource_policy: Some("cross-origin".to_owned()),
            }
        }
    }

    /// The desired resizing method.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, Debug, StringEnum)]
    #[ruma_enum(rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Method {
        /// Crop the original to produce the requested image dimensions.
        Crop,

        /// Maintain the original aspect ratio of the source image.
        Scale,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}
