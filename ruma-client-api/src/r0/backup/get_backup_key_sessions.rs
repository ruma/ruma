//! [GET /_matrix/client/r0/room_keys/keys/{roomId}](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys-roomid)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Retrieve sessions from the backup for a given room.",
        method: GET,
        name: "get_backup_key_sessions",
        path: "/_matrix/client/r0/room_keys/keys/:room_id",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: String,

        /// Room ID.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response: {
        /// A map of session IDs to key data.
        pub sessions: BTreeMap<String, super::KeyData>,
    }

    error: crate::Error
}
