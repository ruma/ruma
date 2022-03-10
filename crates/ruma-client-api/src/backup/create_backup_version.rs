//! `POST /_matrix/client/*/room_keys/version`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3room_keysversion

    use ruma_common::{api::ruma_api, serde::Raw};

    use crate::backup::BackupAlgorithm;

    ruma_api! {
        metadata: {
            description: "Create a new backup version.",
            method: POST,
            name: "create_backup_version",
            unstable_path: "/_matrix/client/unstable/room_keys/version",
            stable_path: "/_matrix/client/v3/room_keys/version",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.1,
        }

        request: {
            /// The algorithm used for storing backups.
            #[ruma_api(body)]
            pub algorithm: Raw<BackupAlgorithm>,
        }

        response: {
            /// The backup version.
            pub version: String,
        }

        error: crate::Error
    }

    impl Request {
        /// Creates a new `Request` with the given backup algorithm.
        pub fn new(algorithm: Raw<BackupAlgorithm>) -> Self {
            Self { algorithm }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given version.
        pub fn new(version: String) -> Self {
            Self { version }
        }
    }
}
