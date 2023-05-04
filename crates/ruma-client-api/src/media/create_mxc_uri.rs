//! `POST /_matrix/media/*/create`
//!
//! Create an MXC URI without content.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/tulir/matrix-doc/blob/asynchronous_uploads/proposals/2246-asynchronous-uploads.md

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, MilliSecondsSinceUnixEpoch, OwnedMxcUri,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/media/unstable/fi.mau.msc2246/create",
        }
    };

    /// Request type for the `create_mxc_uri` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `create_mxc_uri` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The MXC URI for the about to be uploaded content.
        pub content_uri: OwnedMxcUri,

        /// The time at which the URI will expire if an upload has not been started.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub unused_expires_at: Option<MilliSecondsSinceUnixEpoch>,
    }

    impl Response {
        /// Creates a new `Response` with the given MXC URI.
        pub fn new(content_uri: OwnedMxcUri) -> Self {
            Self { content_uri, unused_expires_at: None }
        }
    }
}
