//! [PUT /_matrix/client/r0/directory/list/room/{roomId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-directory-list-room-roomid)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use crate::r0::room::Visibility;

ruma_api! {
    metadata: {
        description: "Set the visibility of a public room on a directory.",
        name: "set_room_visibility",
        method: PUT,
        path: "/_matrix/client/r0/directory/list/room/:room_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The ID of the room of which to set the visibility.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// New visibility setting for the room.
        pub visibility: Visibility,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID and visibility.
    pub fn new(room_id: &'a RoomId, visibility: Visibility) -> Self {
        Self { room_id, visibility }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
