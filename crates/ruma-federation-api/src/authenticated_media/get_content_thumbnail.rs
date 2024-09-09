//! `GET /_matrix/federation/*/media/thumbnail/{mediaId}`
//!
//! Get a thumbnail of content from the media repository.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1mediathumbnailmediaid

    use std::time::Duration;

    use js_int::UInt;
    use ruma_common::{
        api::{request, Metadata},
        media::Method,
        metadata,
    };

    use crate::authenticated_media::{ContentMetadata, FileOrLocation};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/org.matrix.msc3916.v2/media/thumbnail/:media_id",
            1.11 => "/_matrix/federation/v1/media/thumbnail/:media_id",
        }
    };

    /// Request type for the `get_content_thumbnail` endpoint.
    #[request]
    pub struct Request {
        /// The media ID from the `mxc://` URI (the path component).
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

        /// Whether the server should return an animated thumbnail.
        ///
        /// When `Some(true)`, the server should return an animated thumbnail if possible and
        /// supported. When `Some(false)`, the server must not return an animated
        /// thumbnail. When `None`, the server should not return an animated thumbnail.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub animated: Option<bool>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID, desired thumbnail width
        /// and desired thumbnail height.
        pub fn new(media_id: String, width: UInt, height: UInt) -> Self {
            Self {
                media_id,
                method: None,
                width,
                height,
                timeout_ms: ruma_common::media::default_download_timeout(),
                animated: None,
            }
        }
    }

    /// Response type for the `get_content_thumbnail` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Response {
        /// The metadata of the thumbnail.
        pub metadata: ContentMetadata,

        /// The content of the thumbnail.
        pub content: FileOrLocation,
    }

    impl Response {
        /// Creates a new `Response` with the given metadata and content.
        pub fn new(metadata: ContentMetadata, content: FileOrLocation) -> Self {
            Self { metadata, content }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response {
        type EndpointError = ruma_common::api::error::MatrixError;

        fn try_from_http_response<T: AsRef<[u8]>>(
            http_response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::api::EndpointError;

            if http_response.status().as_u16() < 400 {
                let (metadata, content) =
                    crate::authenticated_media::try_from_multipart_mixed_response(http_response)?;
                Ok(Self { metadata, content })
            } else {
                Err(ruma_common::api::error::FromHttpResponseError::Server(
                    ruma_common::api::error::MatrixError::from_http_response(http_response),
                ))
            }
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            crate::authenticated_media::try_into_multipart_mixed_response(
                &self.metadata,
                &self.content,
            )
        }
    }
}
