//! [GET /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get information about an existing backup.",
        method: GET,
        name: "get_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version.
        #[ruma_api(path)]
        pub version: String,
    }

    response: {
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: super::BackupAlgorithm,
        /// The number of keys stored in the backup.
        pub count: UInt,
        /// An opaque string representing stored keys in the backup. Clients can compare it with
        /// the etag value they received in the request of their last key storage request.
        pub etag: String,
        /// The backup version. This is an opaque string.
        pub version: String,
    }

    error: crate::Error
}
