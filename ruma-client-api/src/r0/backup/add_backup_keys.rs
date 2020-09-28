//! [PUT /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#put-matrix-client-r0-room-keys-keys)

use js_int::UInt;
use ruma_api::ruma_api;

use super::Rooms;

ruma_api! {
    metadata: {
        description: "Store several keys in the backup.",
        method: PUT,
        name: "add_backup_keys",
        path: "/_matrix/client/r0/room_keys/keys",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: &'a str,

        /// A map from room IDs to session IDs to key data.
        ///
        /// Note: synapse has the `sessions: {}` wrapper, the Matrix spec does not.
        pub rooms: Rooms,
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
    /// Creates a new `Request` with the given version.
    pub fn new(version: &'a str, rooms: Rooms) -> Self {
        Self { version, rooms }
    }
}

impl Response {
    /// Creates a new `Response` with the given  etag and key count.
    pub fn new(etag: String, count: UInt) -> Self {
        Self { etag, count }
    }
}
