//! `POST /_matrix/media/*/create`
//!
//! Create an MXC URI without content.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/tulir/matrix-doc/blob/asynchronous_uploads/proposals/2246-asynchronous-uploads.md

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri,
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
        pub unused_expires_at: UInt,
    }

    impl Response {
        /// Creates a new `Response` with the given MXC URI which expires at a given point in time.
        pub fn new(content_uri: OwnedMxcUri, unused_expires_at: UInt) -> Self {
            Self { content_uri, unused_expires_at }
        }
    }
}
