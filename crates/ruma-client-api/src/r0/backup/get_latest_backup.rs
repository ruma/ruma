//! [GET /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use js_int::UInt;
use ruma_api::ruma_api;

use super::BackupAlgorithm;

ruma_api! {
    metadata: {
        description: "Get information about the latest backup.",
        method: GET,
        name: "get_latest_backup",
        path: "/_matrix/client/r0/room_keys/version",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: BackupAlgorithm,

        /// The number of keys stored in the backup.
        pub count: UInt,

        /// An opaque string represetning stored keys in the backup. Clients can compare it with
        /// the etag value they received in the request of their last key storage request.
        pub etag: String,

        /// The backup version. This is an opaque string.
        pub version: String,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given algorithm, key count, etag and version.
    pub fn new(algorithm: BackupAlgorithm, count: UInt, etag: String, version: String) -> Self {
        Self { algorithm, count, etag, version }
    }
}
