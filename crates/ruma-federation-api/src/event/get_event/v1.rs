//! [GET /_matrix/federation/v1/event/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-event-eventid)

use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{EventId, ServerName};

use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Retrieves a single event.",
        method: GET,
        name: "get_event",
        path: "/_matrix/federation/v1/event/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The event ID to get.
        #[ruma_api(path)]
        pub event_id: &'a EventId,
    }

    response: {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: Box<ServerName>,

        /// Time on originating homeserver when this transaction started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The event.
        #[serde(rename = "pdus", with = "ruma_serde::single_element_seq")]
        pub pdu: Box<RawJsonValue>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given event id.
    pub fn new(event_id: &'a EventId) -> Self {
        Self { event_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given server name, timestamp, and event.
    pub fn new(
        origin: Box<ServerName>,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        pdu: Box<RawJsonValue>,
    ) -> Self {
        Self { origin, origin_server_ts, pdu }
    }
}
