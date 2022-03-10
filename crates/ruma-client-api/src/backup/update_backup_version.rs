//! `PUT /_matrix/client/*/room_keys/version/{version}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3room_keysversionversion

    use ruma_common::{api::ruma_api, serde::Raw};

    use crate::backup::BackupAlgorithm;

    ruma_api! {
        metadata: {
            description: "Update information about an existing backup.",
            method: PUT,
            name: "update_backup_version",
            unstable_path: "/_matrix/client/unstable/room_keys/version/:version",
            stable_path: "/_matrix/client/v3/room_keys/version/:version",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
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
}
