//! [GET /_matrix/federation/v1/backfill/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-backfill-roomid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{EventId, RoomId, ServerName};

use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Request more history from another homeserver",
        name: "get_backfill",
        method: GET,
        path: "/_matrix/federation/v1/backfill/:room_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID to backfill.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event IDs to backfill from.
        #[ruma_api(query)]
        pub v: &'a [Box<EventId>],

        /// The maximum number of PDUs to retrieve, including the given events.
        #[ruma_api(query)]
        pub limit: UInt,
    }

    response: {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: Box<ServerName>,

        /// POSIX timestamp in milliseconds on originating homeserver when this transaction started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// List of persistent updates to rooms.
        pub pdus: Vec<Box<RawJsonValue>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with:
    /// * the given room id.
    /// * the event IDs to backfill from.
    /// * the maximum number of PDUs to retrieve, including the given events.
    pub fn new(room_id: &'a RoomId, v: &'a [Box<EventId>], limit: UInt) -> Self {
        Self { room_id, v, limit }
    }
}

impl Response {
    /// Creates a new `Response` with:
    /// * the `server_name` of the homeserver.
    /// * the timestamp in milliseconds of when this transaction started.
    /// * the list of persistent updates to rooms.
    pub fn new(
        origin: Box<ServerName>,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        pdus: Vec<Box<RawJsonValue>>,
    ) -> Self {
        Self { origin, origin_server_ts, pdus }
    }
}
