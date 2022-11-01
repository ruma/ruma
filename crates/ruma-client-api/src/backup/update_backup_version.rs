//! `PUT /_matrix/client/*/room_keys/version/{version}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3room_keysversionversion

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
    };

    use crate::backup::BackupAlgorithm;

    const METADATA: Metadata = metadata! {
        description: "Update information about an existing backup.",
        method: PUT,
        name: "update_backup_version",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/version/:version",
            1.1 => "/_matrix/client/v3/room_keys/version/:version",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The backup version.
        #[ruma_api(path)]
        pub version: &'a str,

        /// The algorithm used for storing backups.
        #[ruma_api(body)]
        pub algorithm: Raw<BackupAlgorithm>,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
