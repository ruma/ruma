//! [POST /_matrix/client/r0/rooms/{roomId}/upgrade](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-upgrade)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata {
        description: "Upgrades a room to a particular version.",
        method: POST,
        name: "upgrade_room",
        path: "/_matrix/client/r0/rooms/:room_id/upgrade",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// ID of the room to be upgraded.
        #[ruma_api(path)]
        room_id: RoomId,
        /// New version for the room.
        new_version: String,
    }

    response {
        /// ID of the new room.
        replacement_room: RoomId,
    }
}
