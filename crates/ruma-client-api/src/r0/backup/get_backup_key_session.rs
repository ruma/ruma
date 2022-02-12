//! [GET /_matrix/client/v3/room_keys/keys/{roomId}/{sessionId}](https://spec.matrix.org/v1.1/client-server-api/#get_matrixclientv3room_keyskeysroomidsessionid)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;
use ruma_serde::Raw;

use super::KeyBackupData;

ruma_api! {
    metadata: {
        description: "Retrieve a key from the backup",
        method: GET,
        name: "get_backup_key_session",
        unstable: "/_matrix/client/unstable/room_keys/keys/:room_id/:session_id",
        r0: "/_matrix/client/r0/room_keys/keys/:room_id/:session_id",
        stable: "/_matrix/client/v3/room_keys/keys/:room_id/:session_id",
        rate_limited: true,
        authentication: AccessToken,
        added: 1.0,
    }

    request: {
        /// The backup version.
        ///
        /// Must be the current backup.
        #[ruma_api(query)]
        pub version: &'a str,

        /// The ID of the room that the requested key is for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The ID of the megolm session whose key is requested.
        #[ruma_api(path)]
        pub session_id: &'a str,
    }

    response: {
        /// Information about the requested backup key.
        #[ruma_api(body)]
        pub key_data: Raw<KeyBackupData>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given version, room_id and session_id.
    pub fn new(version: &'a str, room_id: &'a RoomId, session_id: &'a str) -> Self {
        Self { version, room_id, session_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given key_data.
    pub fn new(key_data: Raw<KeyBackupData>) -> Self {
        Self { key_data }
    }
}
