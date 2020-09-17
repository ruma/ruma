//! [PUT /_matrix/client/r0/room_keys/keys/{roomId}](https://matrix.org/docs/spec/client_server/unstable#put-matrix-client-r0-room-keys-keys-roomid)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Store several sessions in the backup.",
        method: PUT,
        name: "add_backup_key_sessions",
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

        /// A map from session IDs to key data.
        pub sessions: BTreeMap<String, super::KeyData>,
    }

    response: {
        /// An opaque string representing stored keys in the backup. Clients can compare it with
        /// the etag value they received in the request of their last key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    error: crate::Error
}
