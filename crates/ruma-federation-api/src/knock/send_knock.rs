//! `PUT /_matrix/federation/*/send_knock/{roomId}/{eventId}`
//!
//! Submits a signed knock event to the resident homeserver for it to accept into the room's graph.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_knockroomideventid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedEventId, OwnedRoomId,
    };
    use ruma_events::AnyStrippedStateEvent;
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/xyz.amorgan.knock/send_knock/{room_id}/{event_id}",
            1.1 => "/_matrix/federation/v1/send_knock/{room_id}/{event_id}",
        }
    };

    /// Request type for the `send_knock` endpoint.
    #[request]
    pub struct Request {
        /// The room ID that should receive the knock.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event ID for the knock event.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// The PDU.
        #[ruma_api(body)]
        pub pdu: Box<RawJsonValue>,
    }

    /// Response type for the `send_knock` endpoint.
    #[response]
    pub struct Response {
        /// State events providing public room metadata.
        pub knock_room_state: Vec<Raw<AnyStrippedStateEvent>>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, event ID and knock event.
        pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, pdu: Box<RawJsonValue>) -> Self {
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
