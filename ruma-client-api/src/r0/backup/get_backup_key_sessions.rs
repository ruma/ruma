//! [GET /_matrix/client/r0/room_keys/keys/{roomId}](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys-roomid)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use super::KeyData;

ruma_api! {
    metadata: {
        description: "Retrieve sessions from the backup for a given room.",
        method: GET,
        name: "get_backup_key_sessions",
        path: "/_matrix/client/r0/room_keys/keys/:room_id",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: &'a str,

        /// The ID of the room that the requested key is for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    response: {
        /// A map of session IDs to key data.
        pub sessions: BTreeMap<String, KeyData>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given version and room_id.
    pub fn new(version: &'a str, room_id: &'a RoomId) -> Self {
        Self { version, room_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given sessions.
    pub fn new(sessions: BTreeMap<String, KeyData>) -> Self {
        Self { sessions }
    }
}
