//! `POST /_matrix/media/*/create`

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/tulir/matrix-doc/blob/asynchronous_uploads/proposals/2246-asynchronous-uploads.md

    use js_int::UInt;
    use ruma_common::{api::ruma_api, OwnedMxcUri};

    ruma_api! {
        metadata: {
            description: "Create an MXC URI without content.",
            method: POST,
            name: "create_mxc_uri",
            unstable_path: "/_matrix/media/unstable/fi.mau.msc2246/create",
            rate_limited: true,
            authentication: AccessToken,
        }

        request: {}

        response: {
            /// The MXC URI for the about to be uploaded content.
            pub content_uri: OwnedMxcUri,

            /// The time at which the URI will expire if an upload has not been started.
            pub unused_expires_at: UInt,
        }

        error: crate::Error
    }

    impl Response {
        /// Creates a new `Response` with the given MXC URI which expires at a given point in time.
        pub fn new(content_uri: OwnedMxcUri, unused_expires_at: UInt) -> Self {
            Self { content_uri, unused_expires_at }
        }
    }
}
