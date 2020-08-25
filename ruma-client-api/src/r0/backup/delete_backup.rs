//! [DELETE /_matrix/client/r0/room_keys/version/{version}](https://matrix.org/docs/spec/client_server/unstable#delete-matrix-client-r0-room-keys-version-version)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Delete an existing backup.",
        method: DELETE,
        name: "delete_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version.
        #[ruma_api(path)]
        pub version: String,
    }

    response: {}

    error: crate::Error
}
