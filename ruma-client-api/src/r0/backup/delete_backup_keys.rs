//! [DELETE /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#delete-matrix-client-r0-room-keys-keys)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Delete all keys in a backup.",
        method: PUT,
        name: "delete_backup_keys",
        path: "/_matrix/client/r0/room_keys/keys",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: String,
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
