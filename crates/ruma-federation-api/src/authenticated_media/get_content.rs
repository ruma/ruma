//! `GET /_matrix/federation/*/media/download/{mediaId}`
//!
//! Retrieve content from the media store.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1mediadownloadmediaid

    use std::time::Duration;

    use ruma_common::{
        api::{request, Metadata},
        metadata,
    };

    use crate::authenticated_media::{ContentMetadata, FileOrLocation};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/org.matrix.msc3916.v2/media/download/:media_id",
            1.11 => "/_matrix/federation/v1/media/download/:media_id",
        }
    };

    /// Request type for the `get_content` endpoint.
    #[request]
    pub struct Request {
        /// The media ID from the `mxc://` URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

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
    }

    impl Request {
        /// Creates a new `Request` with the given media ID.
        pub fn new(media_id: String) -> Self {
            Self { media_id, timeout_ms: ruma_common::media::default_download_timeout() }
        }
    }

    /// Response type for the `get_content` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Response {
        /// The metadata of the media.
        pub metadata: ContentMetadata,

        /// The content of the media.
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
