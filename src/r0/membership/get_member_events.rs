//! [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-members)

use ruma_api::ruma_api;
use ruma_events::{room::member::MemberEvent, EventResult};
use ruma_identifiers::RoomId;

ruma_api! {
    metadata {
        description: "Get membership events for a room.",
        method: GET,
        name: "get_member_events",
        path: "/_matrix/client/r0/rooms/:room_id/members",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to get the member events for.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {
        /// A list of member events.
        #[wrap_incoming(MemberEvent with EventResult)]
        pub chunk: Vec<MemberEvent>
    }

    error: crate::Error
}
