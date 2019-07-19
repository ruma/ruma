//! [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-members)

use ruma_api::ruma_api;
use ruma_events::room::member::MemberEvent;
use ruma_identifiers::RoomId;
use serde::Deserialize;

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
        pub chunk: Vec<MemberEvent>
    }
}
