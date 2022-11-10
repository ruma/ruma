//! `PUT /_matrix/federation/*/send_knock/{roomId}/{eventId}`
//!
//! Submits a signed knock event to the resident homeserver for it to accept into the room's graph.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#put_matrixfederationv1send_knockroomideventid

    use ruma_common::{
        api::{request, response, Metadata},
        events::AnyStrippedStateEvent,
        metadata,
        serde::Raw,
        EventId, RoomId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/xyz.amorgan.knock/send_knock/:room_id/:event_id",
            1.1 => "/_matrix/federation/v1/send_knock/:room_id/:event_id",
        }
    };

    /// Request type for the `send_knock` endpoint.
    #[request]
    pub struct Request<'a> {
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

    /// Response type for the `send_knock` endpoint.
    #[response]
    pub struct Response {
        /// State events providing public room metadata.
        pub knock_room_state: Vec<Raw<AnyStrippedStateEvent>>,
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
