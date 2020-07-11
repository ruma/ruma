//! [POST /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Update information about an existing backup.",
        method: POST,
        name: "update_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version.
        #[ruma_api(path)]
        pub version: String,
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: super::BackupAlgorithm,
    }

    response: {}

    error: crate::Error
}
