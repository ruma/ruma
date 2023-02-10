//! `GET /_matrix/client/*/room_keys/keys`
//!
//! Retrieve all keys from a backup version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3room_keyskeys

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::backup::RoomKeyBackup;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys",
            1.0 => "/_matrix/client/r0/room_keys/keys",
            1.1 => "/_matrix/client/v3/room_keys/keys",
        }
    };

    /// Request type for the `get_backup_keys` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version to retrieve keys from.
        #[ruma_api(query)]
        pub version: String,
    }

    /// Response type for the `get_backup_keys` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A map from room IDs to session IDs to key data.
        pub rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>,
    }

    impl Request {
        /// Creates a new `Request` with the given version.
        pub fn new(version: String) -> Self {
            Self { version }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room key backups.
        pub fn new(rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>) -> Self {
            Self { rooms }
        }
    }
}
