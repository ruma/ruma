//! [DELETE /_matrix/client/r0/room_keys/keys/{roomId}](https://matrix.org/docs/spec/client_server/unstable#delete-matrix-client-r0-room-keys-keys-roomid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Delete keys from the backup for a given room.",
        method: GET,
        name: "delete_backup_key_sessions",
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
        /// An opaque string representing stored keys in the backup. Clients can compare it with
        /// the etag value they received in the request of their last key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
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
    /// Creates an new `Response` with the given etag and count.
    pub fn new(etag: String, count: UInt) -> Self {
        Self { etag, count }
    }
}
