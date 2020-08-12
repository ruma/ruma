//! [PUT /_matrix/federation/v2/send_join/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v2-send-join-roomid-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::PduStub;
use ruma_common::Raw;
use ruma_identifiers::{EventId, RoomId};

use super::RoomState;

ruma_api! {
    metadata: {
        description: "Send a join event to a resident server.",
        name: "create_join_event",
        method: PUT,
        path: "/_matrix/federation/v2/send_join/:room_id/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub event_id: EventId,

        /// PDU type without event and room IDs.
        #[ruma_api(body)]
        pub pdu_stub: Raw<PduStub>,
    }

    response: {
        /// Full state of the room.
        #[ruma_api(body)]
        pub room_state: RoomState,
    }
}
