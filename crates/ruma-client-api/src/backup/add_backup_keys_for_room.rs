//! `PUT /_matrix/client/*/room_keys/keys/{roomId}`
//!
//! Store keys in the backup for a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3room_keyskeysroomid

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId,
    };

    use crate::backup::KeyBackupData;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys/{room_id}",
            1.0 => "/_matrix/client/r0/room_keys/keys/{room_id}",
            1.1 => "/_matrix/client/v3/room_keys/keys/{room_id}",
        }
    };

    /// Request type for the `add_backup_keys_for_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version to add keys to.
        ///
        /// Must be the current backup.
        #[ruma_api(query)]
        pub version: String,

        /// The ID of the room to add keys to.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// A map of session IDs to key data to store.
        pub sessions: BTreeMap<String, Raw<KeyBackupData>>,
    }

    /// Response type for the `add_backup_keys_for_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// An opaque string representing stored keys in the backup.
        ///
        /// Clients can compare it with the etag value they received in the request of their last
        /// key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    impl Request {
        /// Creates a new `Request` with the given version, room_id and sessions.
        pub fn new(
            version: String,
            room_id: OwnedRoomId,
            sessions: BTreeMap<String, Raw<KeyBackupData>>,
        ) -> Self {
            Self { version, room_id, sessions }
        }
    }

    impl Response {
        /// Creates an new `Response` with the given etag and count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
