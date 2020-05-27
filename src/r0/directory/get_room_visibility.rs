//! [GET /_matrix/client/r0/directory/list/room/{roomId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-directory-list-room-roomid)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use crate::r0::room::Visibility;

ruma_api! {
    metadata {
        description: "Get the visibility of a public room on a directory.",
        name: "get_room_visibility",
        method: GET,
        path: "/_matrix/client/r0/directory/list/room/:room_id",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// The ID of the room of which to request the visibility.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {
        /// Visibility of the room.
        pub visibility: Visibility,
    }

    error: crate::Error
}
