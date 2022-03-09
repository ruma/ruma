//! `GET /_matrix/client/*/room_keys/keys`
//!
//! Retrieve all keys from a backup version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3room_keyskeys

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, RoomId};

    use crate::backup::RoomKeyBackup;

    ruma_api! {
        metadata: {
            description: "Retrieve all keys from a backup version.",
            method: GET,
            name: "get_backup_keys",
            unstable_path: "/_matrix/client/unstable/room_keys/keys",
            r0_path: "/_matrix/client/r0/room_keys/keys",
            stable_path: "/_matrix/client/v3/room_keys/keys",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version to retrieve keys from.
            #[ruma_api(query)]
            pub version: &'a str,
        }

        response: {
            /// A map from room IDs to session IDs to key data.
            pub rooms: BTreeMap<Box<RoomId>, RoomKeyBackup>,
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
        /// Creates a new `Response` with the given room key backups.
        pub fn new(rooms: BTreeMap<Box<RoomId>, RoomKeyBackup>) -> Self {
            Self { rooms }
        }
    }
}
