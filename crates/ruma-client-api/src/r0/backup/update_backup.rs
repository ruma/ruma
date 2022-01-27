//! [POST /_matrix/client/r0/room_keys/version](https://matrix.org/docs/spec/client_server/unstable#post-matrix-client-r0-room-keys-version)

use ruma_api::ruma_api;
use ruma_serde::Raw;

use super::BackupAlgorithm;

ruma_api! {
    metadata: {
        description: "Update information about an existing backup.",
        method: POST,
        name: "update_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version.
        #[ruma_api(path)]
        pub version: &'a str,

        /// The algorithm used for storing backups.
        #[ruma_api(body)]
        pub algorithm: Raw<BackupAlgorithm>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given backup version and algorithm.
    pub fn new(version: &'a str, algorithm: Raw<BackupAlgorithm>) -> Self {
        Self { version, algorithm }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
