//! `PUT /_matrix/federation/*/send_knock/{roomId}/{eventId}`
//!
//! Endpoint to submit a signed knock event to the resident homeserver.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1send_knockroomideventid

    use ruma_common::{api::ruma_api, events::AnyStrippedStateEvent, serde::Raw, EventId, RoomId};
    use serde_json::value::RawValue as RawJsonValue;

    ruma_api! {
        metadata: {
            description: "Submits a signed knock event to the resident homeserver for it to accept into the room's graph.",
            name: "send_knock",
            method: PUT,
            unstable_path: "/_matrix/federation/unstable/xyz.amorgan.knock/send_knock/:room_id/:event_id",
            stable_path: "/_matrix/federation/v1/send_knock/:room_id/:event_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.1,
        }

        request: {
            /// The room ID that should receive the knock.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The event ID for the knock event.
            #[ruma_api(path)]
            pub event_id: &'a EventId,

            /// The PDU.
            #[ruma_api(body)]
            pub pdu: &'a RawJsonValue,
        }

        response: {
            /// State events providing public room metadata.
            pub knock_room_state: Vec<Raw<AnyStrippedStateEvent>>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID, event ID and knock event.
        pub fn new(room_id: &'a RoomId, event_id: &'a EventId, pdu: &'a RawJsonValue) -> Self {
            Self { room_id, event_id, pdu }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given public room metadata state events.
        pub fn new(knock_room_state: Vec<Raw<AnyStrippedStateEvent>>) -> Self {
            Self { knock_room_state }
        }
    }
}
