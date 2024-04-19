//! `GET /_matrix/media/*/thumbnail/{serverName}/{mediaId}`
//!
//! Get a thumbnail of content from the media store.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3thumbnailservernamemediaid

    use std::time::Duration;

    use http::header::CONTENT_TYPE;
    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::StringEnum,
        IdParseError, MxcUri, OwnedServerName,
    };

    use crate::{http_headers::CROSS_ORIGIN_RESOURCE_POLICY, PrivOwnedStr};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: None,
        history: {
            1.0 => "/_matrix/media/r0/thumbnail/:server_name/:media_id",
            1.1 => "/_matrix/media/v3/thumbnail/:server_name/:media_id",
        }
    };

    /// Request type for the `get_content_thumbnail` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

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

        /// Whether the server should return an animated thumbnail.
        ///
        /// When `true`, the server should return an animated thumbnail if possible and supported.
        /// Otherwise it must not return an animated thumbnail.
        ///
        /// Defaults to `false`.
        #[cfg(feature = "unstable-msc2705")]
        #[ruma_api(query)]
        #[serde(
            rename = "org.matrix.msc2705.animated",
            default,
            skip_serializing_if = "ruma_common::serde::is_default"
        )]
        pub animated: bool,
    }

    /// Response type for the `get_content_thumbnail` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
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

    impl Request {
        /// Creates a new `Request` with the given media ID, server name, desired thumbnail width
        /// and desired thumbnail height.
        pub fn new(
            media_id: String,
            server_name: OwnedServerName,
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
                timeout_ms: crate::media::default_download_timeout(),
                allow_redirect: false,
                #[cfg(feature = "unstable-msc2705")]
                animated: false,
            }
        }

        /// Creates a new `Request` with the given url, desired thumbnail width and
        /// desired thumbnail height.
        pub fn from_url(url: &MxcUri, width: UInt, height: UInt) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned(), width, height))
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
    #[derive(Clone, StringEnum)]
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
