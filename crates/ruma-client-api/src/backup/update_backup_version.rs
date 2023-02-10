//! `PUT /_matrix/client/*/room_keys/version/{version}`
//!
//! Update information about an existing backup.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3room_keysversionversion

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
    };

    use crate::backup::BackupAlgorithm;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/version/:version",
            1.1 => "/_matrix/client/v3/room_keys/version/:version",
        }
    };

    /// Request type for the `update_backup_version` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version.
        #[ruma_api(path)]
        pub version: String,

        /// The algorithm used for storing backups.
        #[ruma_api(body)]
        pub algorithm: Raw<BackupAlgorithm>,
    }

    /// Response type for the `update_backup_version` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given backup version and algorithm.
        pub fn new(version: String, algorithm: Raw<BackupAlgorithm>) -> Self {
            Self { version, algorithm }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
