//! `GET /_matrix/client/*/rooms/{roomId}/timestamp_to_event`
//!
//! Get the ID of the event closest to the given timestamp.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1roomsroomidtimestamp_to_event

    use ruma_common::{
        api::{request, response, Direction, Metadata},
        metadata, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3030/rooms/{room_id}/timestamp_to_event",
            1.6 => "/_matrix/client/v1/rooms/{room_id}/timestamp_to_event",
        }
    };

    /// Request type for the `get_event_by_timestamp` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room the event is in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The timestamp to search from, inclusively.
        #[ruma_api(query)]
        pub ts: MilliSecondsSinceUnixEpoch,

        /// The direction in which to search.
        #[ruma_api(query)]
        pub dir: Direction,
    }

    /// Response type for the `get_event_by_timestamp` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The ID of the event found.
        pub event_id: OwnedEventId,

        /// The event's timestamp.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, timestamp and direction.
        pub fn new(room_id: OwnedRoomId, ts: MilliSecondsSinceUnixEpoch, dir: Direction) -> Self {
            Self { room_id, ts, dir }
        }

        /// Creates a new `Request` with the given room ID and timestamp, and the direction set to
        /// `Backward`.
        ///
        /// Allows to have the latest event before or including the given timestamp.
        pub fn until(room_id: OwnedRoomId, ts: MilliSecondsSinceUnixEpoch) -> Self {
            Self::new(room_id, ts, Direction::Backward)
        }

        /// Creates a new `Request` with the given room ID and timestamp, and the direction set to
        /// `Forward`.
        ///
        /// Allows to have the earliest event including or after the given timestamp.
        pub fn since(room_id: OwnedRoomId, ts: MilliSecondsSinceUnixEpoch) -> Self {
            Self::new(room_id, ts, Direction::Forward)
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event ID and timestamp.
        pub fn new(event_id: OwnedEventId, origin_server_ts: MilliSecondsSinceUnixEpoch) -> Self {
            Self { event_id, origin_server_ts }
        }
    }
}
