//! [POST /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use ruma_api::ruma_api;

use super::BackupAlgorithm;

ruma_api! {
    metadata: {
        description: "Creates a new backup.",
        method: POST,
        name: "create_backup",
        path: "/_matrix/client/r0/room_keys/version",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The algorithm used for storing backups.
        #[serde(flatten)]
        pub algorithm: BackupAlgorithm,
    }

    response: {
        /// The backup version. This is an opaque string.
        pub version: String,
    }

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given backup algorithm.
    pub fn new(algorithm: BackupAlgorithm) -> Self {
        Self { algorithm }
    }
}

impl Response {
    /// Creates a new `Response` with the given version.
    pub fn new(version: String) -> Self {
        Self { version }
    }
}
