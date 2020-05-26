//! [PUT /_matrix/client/r0/directory/list/room/{roomId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-directory-list-room-roomid)

use ruma_api::ruma_api;

use crate::r0::room::Visibility;

ruma_api! {
    metadata {
        description: "Set the visibility of a public room on a directory.",
        name: "set_room_visibility",
        method: PUT,
        path: "/_matrix/client/r0/directory/list/room/:room_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The ID of the room of which to set the visibility.
        #[ruma_api(path)]
        pub room_id: String,

        /// New visibility setting for the room.
        pub visibility: Visibility,
    }

    response {}

    error: crate::Error
}
