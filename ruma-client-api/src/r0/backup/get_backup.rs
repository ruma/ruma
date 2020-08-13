//! [GET /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use js_int::UInt;
use ruma_api::ruma_api;

use super::BackupAlgorithm;

ruma_api! {
    metadata: {
        description: "Get information about an existing backup.",
        method: GET,
        name: "get_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version.
        #[ruma_api(path)]
        pub version: &'a str,
    }

    response: {
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: BackupAlgorithm,

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

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given version.
    pub fn new(version: &'a str) -> Self {
        Self { version }
    }
}

impl Response {
    /// Creates a new `Response` with the gien algorithm, key count, etag and version.
    pub fn new(algorithm: BackupAlgorithm, count: UInt, etag: String, version: String) -> Self {
        Self { algorithm, count, etag, version }
    }
}
