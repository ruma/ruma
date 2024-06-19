//! `POST /_matrix/federation/*/get_missing_events/{roomId}`
//!
//! Retrieves previous events that the sender is missing.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixfederationv1get_missing_eventsroomid

    use js_int::{uint, UInt};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/get_missing_events/{room_id}",
        }
    };

    /// Request type for the `get_missing_events` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to search in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The maximum number of events to retrieve.
        ///
        /// Defaults to 10.
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// The minimum depth of events to retrieve.
        ///
        /// Defaults to 0.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub min_depth: UInt,

        /// The latest event IDs that the sender already has.
        ///
        /// These are skipped when retrieving the previous events of `latest_events`.
        pub earliest_events: Vec<OwnedEventId>,

        /// The event IDs to retrieve the previous events for.
        pub latest_events: Vec<OwnedEventId>,
    }

    /// Response type for the `get_missing_events` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The missing PDUs.
        pub events: Vec<Box<RawJsonValue>>,
    }

    impl Request {
        /// Creates a new `Request` for events in the given room with the given constraints.
        pub fn new(
            room_id: OwnedRoomId,
            earliest_events: Vec<OwnedEventId>,
            latest_events: Vec<OwnedEventId>,
        ) -> Self {
            Self {
                room_id,
                limit: default_limit(),
                min_depth: UInt::default(),
                earliest_events,
                latest_events,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given events.
        pub fn new(events: Vec<Box<RawJsonValue>>) -> Self {
            Self { events }
        }
    }

    fn default_limit() -> UInt {
        uint!(10)
    }

    fn is_default_limit(val: &UInt) -> bool {
        *val == default_limit()
    }
}
