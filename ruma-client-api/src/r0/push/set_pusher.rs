//! [POST /_matrix/client/r0/pushers/set](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-pushers-set)

use ruma_api::ruma_api;

use super::Pusher;

ruma_api! {
    metadata: {
        description: "This endpoint allows the creation, modification and deletion of pushers for this user ID.",
        method: POST,
        name: "set_pusher",
        path: "/_matrix/client/r0/pushers/set",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The pusher to configure.
        #[serde(flatten)]
        pub pusher: Pusher,

        /// Controls if another pusher with the same pushkey and app id should be created.
        ///
        /// Defaults to `false`. See the spec for more details.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        pub append: bool,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given pusher.
    pub fn new(pusher: Pusher) -> Self {
        Self { pusher, append: false }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
