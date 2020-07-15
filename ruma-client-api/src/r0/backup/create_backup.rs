//! [POST /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Creates a new backup.",
        method: POST,
        name: "create_backup",
        path: "/_matrix/client/r0/room_keys/version",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: super::BackupAlgorithm,
    }

    response: {
        /// The backup version. This is an opaque string.
        pub version: String,
    }

    error: crate::Error
}
