//! `PUT /_matrix/client/*/room_keys/keys`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3room_keyskeys

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{api::ruma_api, OwnedRoomId};

    use crate::backup::RoomKeyBackup;

    ruma_api! {
        metadata: {
            description: "Store keys in the backup.",
            method: PUT,
            name: "add_backup_keys",
            unstable_path: "/_matrix/client/unstable/room_keys/keys",
            stable_path: "/_matrix/client/v3/room_keys/keys",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.1,
        }

        request: {
            /// The backup version to add keys to.
            ///
            /// Must be the current backup.
            #[ruma_api(query)]
            pub version: &'a str,

            /// A map of room IDs to session IDs to key data to store.
            pub rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>,
        }

        response: {
            /// An opaque string representing stored keys in the backup.
            ///
            /// Clients can compare it with  the etag value they received in the request of their last
            /// key storage request.
            pub etag: String,

            /// The number of keys stored in the backup.
            pub count: UInt,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version and room key backups.
        pub fn new(version: &'a str, rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>) -> Self {
            Self { version, rooms }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given  etag and key count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
