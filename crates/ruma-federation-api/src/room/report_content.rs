//! `GET /_matrix/federation/*/rooms/{roomId}/report/{eventId}`
//!
//! Send a request to report an event originating from another server.

pub mod msc3843 {
    //! `MSC3843` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3843

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/org.matrix.msc3843/rooms/{room_id}/report/{event_id}",
        }
    };

    /// Request type for the `report_content` endpoint.
    #[request]
    pub struct Request {
        /// The room ID that the reported event was sent in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event being reported.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// The reason that the event is being reported.
        pub reason: String,
    }

    /// Response type for the `report_content` endpoint.
    #[response]
    pub struct Response {}

    impl Request {
        /// Creates a `Request` with the given room ID, event ID and reason.
        pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, reason: String) -> Self {
            Self { room_id, event_id, reason }
        }
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
