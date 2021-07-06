//! [POST /_matrix/client/r0/rooms/{roomId}/leave](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-leave)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Leave a room.",
        method: POST,
        name: "leave_room",
        path: "/_matrix/client/r0/rooms/:room_id/leave",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The room to leave.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// Optional reason to be included as the `reason` on the subsequent membership event.
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<&'a str>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id.
    pub fn new(room_id: &'a RoomId) -> Self {
        Self {
            room_id,
            #[cfg(feature = "unstable-pre-spec")]
            reason: None,
        }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
