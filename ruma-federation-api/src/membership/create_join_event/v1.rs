//! [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-join-roomid-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

use super::RoomState;

ruma_api! {
    metadata: {
        description: "Send a join event to a resident server.",
        name: "create_join_event",
        method: PUT,
        path: "/_matrix/federation/v1/send_join/:room_id/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID that is about to be joined.
        ///
        /// Do not use this. Instead, use the `room_id` field inside the PDU.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event ID for the join event.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The PDU.
        #[ruma_api(body)]
        pub pdu: Raw<Pdu>,
    }

    response: {
        /// Full state and auth chain of the room prior to the join event.
        #[ruma_api(body)]
        #[serde(with = "crate::serde::v1_pdu")]
        pub room_state: RoomState,
    }
}

impl<'a> Request<'a> {
    /// Creates a `Request` from the given room ID, event ID and `Pdu`.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId, pdu: Raw<Pdu>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Response {
    /// Creates a new `Response` with the given room state.
    pub fn new(room_state: RoomState) -> Self {
        Self { room_state }
    }
}
