//! [POST /_matrix/media/r0/upload](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-media-r0-upload)

use ruma_api_macros::ruma_api;
use serde_derive::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Upload content to the media store.",
        method: POST,
        name: "create_media_content",
        path: "/_matrix/media/r0/upload",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// The content type of the file being uploaded.
        #[ruma_api(header = "CONTENT_TYPE")]
        pub content_type: String,
    }

    response {
        /// The MXC URI for the uploaded content.
        pub content_uri: String,
    }
}
