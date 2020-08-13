//! [POST /_matrix/client/r0/rooms/{roomId}/upgrade](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-upgrade)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, RoomVersionId};

ruma_api! {
    metadata: {
        description: "Upgrades a room to a particular version.",
        method: POST,
        name: "upgrade_room",
        path: "/_matrix/client/r0/rooms/:room_id/upgrade",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// ID of the room to be upgraded.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// New version for the room.
        pub new_version: &'a RoomVersionId,
    }

    response: {
        /// ID of the new room.
        pub replacement_room: RoomId,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID and new room version.
    pub fn new(room_id: &'a RoomId, new_version: &'a RoomVersionId) -> Self {
        Self { room_id, new_version }
    }
}

impl Response {
    /// Creates a new `Response` with the given room ID.
    pub fn new(replacement_room: RoomId) -> Self {
        Self { replacement_room }
    }
}
