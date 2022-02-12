//! [GET /_matrix/client/v3/room_keys/keys](https://spec.matrix.org/v1.1/client-server-api/#get_matrixclientv3room_keyskeys)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use super::RoomKeyBackup;

ruma_api! {
    metadata: {
        description: "Retrieve all keys from a backup.",
        method: GET,
        name: "get_backup_keys",
        path: "/_matrix/client/unstable/room_keys/keys",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version.
        ///
        /// Must be the current backup.
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
