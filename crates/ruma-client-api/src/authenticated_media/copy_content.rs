//! `POST /_matrix/client/*/media/copy/{serverName}/{mediaId}`
//!
//! Generate a new media ID for content already in the media store

pub mod unstable {
    //! `/unstable/org.matrix.msc3911/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3911

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3911/media/copy/:server_name/:media_id",
        }
    };

    /// Request type for the `copy_content` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {}

    /// Response type for the `copy_content` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The newly generated MXC URI for the copied content
        pub content_uri: OwnedMxcUri,
    }

    impl Request {
        /// Creates a new empty `Request`
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given MXC URI.
        pub fn new(content_uri: OwnedMxcUri) -> Self {
            Self { content_uri }
        }
    }
}
