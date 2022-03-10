//! `GET /_matrix/client/*/rooms/{roomId}/event/{eventId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomideventeventid

    use ruma_common::{api::ruma_api, events::AnyRoomEvent, serde::Raw, EventId, RoomId};

    ruma_api! {
        metadata: {
            description: "Get a single event based on roomId/eventId",
            method: GET,
            name: "get_room_event",
            r0_path: "/_matrix/client/r0/rooms/:room_id/event/:event_id",
            stable_path: "/_matrix/client/v3/rooms/:room_id/event/:event_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The ID of the room the event is in.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The ID of the event.
            #[ruma_api(path)]
            pub event_id: &'a EventId,
        }

        response: {
            /// Arbitrary JSON of the event body.
            #[ruma_api(body)]
            pub event: Raw<AnyRoomEvent>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and event ID.
        pub fn new(room_id: &'a RoomId, event_id: &'a EventId) -> Self {
            Self { room_id, event_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event.
        pub fn new(event: Raw<AnyRoomEvent>) -> Self {
            Self { event }
        }
    }
}
