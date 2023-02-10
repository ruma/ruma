//! `PUT /_matrix/client/*/room_keys/keys`
//!
//! Store keys in the backup.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3room_keyskeys

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::backup::RoomKeyBackup;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys",
            1.1 => "/_matrix/client/v3/room_keys/keys",
        }
    };

    /// Request type for the `add_backup_keys` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version to add keys to.
        ///
        /// Must be the current backup.
        #[ruma_api(query)]
        pub version: String,

        /// A map of room IDs to session IDs to key data to store.
        pub rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>,
    }

    /// Response type for the `add_backup_keys` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// An opaque string representing stored keys in the backup.
        ///
        /// Clients can compare it with  the etag value they received in the request of their last
        /// key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    impl Request {
        /// Creates a new `Request` with the given version and room key backups.
        pub fn new(version: String, rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>) -> Self {
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
