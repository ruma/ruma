//! [GET /_matrix/client/r0/room_keys/keys/{roomId}/{sessionId}](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys-roomid-sessionid)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Retrieve a key from the backup",
        method: GET,
        name: "get_backup_key_session",
        path: "/_matrix/client/r0/room_keys/keys/:room_id/:session_id",
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
        /// Session ID.
        #[ruma_api(path)]
        pub session_id: String,
    }

    response: {
        /// Key data.
        #[ruma_api(body)]
        pub key_data: Option<super::KeyData>,
    }

    error: crate::Error
}
