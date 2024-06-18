//! `GET /_matrix/federation/*/event_auth/{roomId}/{eventId}`
//!
//! Get the complete auth chain for a given event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1event_authroomideventid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/event_auth/{room_id}/{event_id}",
        }
    };

    /// Request type for the `get_event_authorization` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to get the auth chain for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event ID to get the auth chain for.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,
    }

    /// Response type for the `get_event_authorization` endpoint.
    #[response]
    pub struct Response {
        /// The full set of authorization events that make up the state of the room,
        /// and their authorization events, recursively.
        pub auth_chain: Vec<Box<RawJsonValue>>,
    }

    impl Request {
        /// Creates a new `Request` with the given room id and event id.
        pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId) -> Self {
            Self { room_id, event_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given auth chain.
        pub fn new(auth_chain: Vec<Box<RawJsonValue>>) -> Self {
            Self { auth_chain }
        }
    }
}
