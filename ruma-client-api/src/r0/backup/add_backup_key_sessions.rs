//! [PUT /_matrix/client/r0/room_keys/keys/{roomId}](https://matrix.org/docs/spec/client_server/unstable#put-matrix-client-r0-room-keys-keys-roomid)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use super::KeyData;

ruma_api! {
    metadata: {
        description: "Store several sessions in the backup.",
        method: PUT,
        name: "add_backup_key_sessions",
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

        /// A map from session IDs to key data.
        pub sessions: BTreeMap<String, KeyData>,
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

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given version, room_id and sessions.
    pub fn new(version: &'a str, room_id: &'a RoomId, sessions: BTreeMap<String, KeyData>) -> Self {
        Self { version, room_id, sessions }
    }
}

impl Response {
    /// Creates an new `Response` with the given etag and count.
    pub fn new(etag: String, count: UInt) -> Self {
        Self { etag, count }
    }
}
