//! [POST /_matrix/client/r0/pushers/set](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-pushers-set)

use ruma_api::ruma_api;

use super::Pusher;

ruma_api! {
    metadata {
        description: "This endpoint allows the creation, modification and deletion of pushers for this user ID.",
        method: POST,
        name: "set_pusher",
        path: "/_matrix/client/r0/pushers/set",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The pusher to configure
        #[serde(flatten)]
        pub pusher: Pusher,

        /// Controls if another pusher with the same pushkey and app id should be created.
        /// See the spec for details.
        #[serde(default)]
        pub append: bool

    }

    response {}

    error: crate::Error
}
