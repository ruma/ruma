//! [GET /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys)

use ruma_api::ruma_api;

use super::Rooms;

ruma_api! {
    metadata: {
        description: "Retrieve all keys from a backup.",
        method: GET,
        name: "get_backup_keys",
        path: "/_matrix/client/r0/room_keys/keys",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: &'a str,
    }

    response: {
        /// A map from room IDs to session IDs to key data.
        ///
        /// Note: synapse has the `sessions: {}` wrapper, the Matrix spec does not.
        pub rooms: Rooms,
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
    /// Creates a new `Response` with the given rooms.
    pub fn new(rooms: Rooms) -> Self {
        Self { rooms }
    }
}
