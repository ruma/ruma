//! `/unstable/org.matrix.msc3916.v2/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916

use std::time::Duration;

use ruma_common::api::{Metadata, path_builder::SinglePath, request};

use crate::authenticated_media::{ContentMetadata, FileOrLocation};

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

impl Metadata for Request {
    const METHOD: http::Method = super::v1::Request::METHOD;
    const RATE_LIMITED: bool = super::v1::Request::RATE_LIMITED;
    type Authentication = <super::v1::Request as Metadata>::Authentication;
    type PathBuilder = <super::v1::Request as Metadata>::PathBuilder;
    const PATH_BUILDER: Self::PathBuilder = SinglePath::new(
        "/_matrix/federation/unstable/org.matrix.msc3916.v2/media/download/{media_id}",
    );
}

impl From<super::v1::Request> for Request {
    fn from(value: super::v1::Request) -> Self {
        let super::v1::Request { media_id, timeout_ms } = value;
        Self { media_id, timeout_ms }
    }
}

impl From<Request> for super::v1::Request {
    fn from(value: Request) -> Self {
        let Request { media_id, timeout_ms } = value;
        Self { media_id, timeout_ms }
    }
}

/// Response type for the `get_content` endpoint.
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
    type EndpointError = <super::v1::Response as ruma_common::api::IncomingResponse>::EndpointError;

    fn try_from_http_response<T: AsRef<[u8]>>(
        http_response: http::Response<T>,
    ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>> {
        // Reuse the custom deserialization.
        Ok(super::v1::Response::try_from_http_response(http_response)?.into())
    }
}

#[cfg(feature = "server")]
impl ruma_common::api::OutgoingResponse for Response {
    fn try_into_http_response<T: Default + bytes::BufMut>(
        self,
    ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
        // Reuse the custom serialization.
        super::v1::Response::from(self).try_into_http_response()
    }
}

impl From<super::v1::Response> for Response {
    fn from(value: super::v1::Response) -> Self {
        let super::v1::Response { metadata, content } = value;
        Self { metadata, content }
    }
}

impl From<Response> for super::v1::Response {
    fn from(value: Response) -> Self {
        let Response { metadata, content } = value;
        Self { metadata, content }
    }
}
