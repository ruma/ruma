//! `GET /_matrix/client/*/room_keys/keys`
//!
//! Retrieve all keys from a backup version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3room_keyskeys

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::backup::RoomKeyBackup;

    const METADATA: Metadata = metadata! {
        description: "Retrieve all keys from a backup version.",
        method: GET,
        name: "get_backup_keys",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys",
            1.0 => "/_matrix/client/r0/room_keys/keys",
            1.1 => "/_matrix/client/v3/room_keys/keys",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The backup version to retrieve keys from.
        #[ruma_api(query)]
        pub version: &'a str,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// A map from room IDs to session IDs to key data.
        pub rooms: BTreeMap<OwnedRoomId, RoomKeyBackup>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version.
        pub fn new(version: &'a str) -> Self {
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
