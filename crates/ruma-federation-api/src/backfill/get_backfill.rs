//! `GET /_matrix/federation/*/backfill/{roomId}`
//!
//! Get more history from another homeserver.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1backfillroomid

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedServerName,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/backfill/{room_id}",
        }
    };

    /// Request type for the `get_backfill` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to backfill.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event IDs to backfill from.
        #[ruma_api(query)]
        pub v: Vec<OwnedEventId>,

        /// The maximum number of PDUs to retrieve, including the given events.
        #[ruma_api(query)]
        pub limit: UInt,
    }

    /// Response type for the `get_backfill` endpoint.
    #[response]
    pub struct Response {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: OwnedServerName,

        /// POSIX timestamp in milliseconds on originating homeserver when this transaction
        /// started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// List of persistent updates to rooms.
        pub pdus: Vec<Box<RawJsonValue>>,
    }

    impl Request {
        /// Creates a new `Request` with:
        /// * the given room id.
        /// * the event IDs to backfill from.
        /// * the maximum number of PDUs to retrieve, including the given events.
        pub fn new(room_id: OwnedRoomId, v: Vec<OwnedEventId>, limit: UInt) -> Self {
            Self { room_id, v, limit }
        }
    }

    impl Response {
        /// Creates a new `Response` with:
        /// * the `server_name` of the homeserver.
        /// * the timestamp in milliseconds of when this transaction started.
        /// * the list of persistent updates to rooms.
        pub fn new(
            origin: OwnedServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
            pdus: Vec<Box<RawJsonValue>>,
        ) -> Self {
            Self { origin, origin_server_ts, pdus }
        }
    }
}
