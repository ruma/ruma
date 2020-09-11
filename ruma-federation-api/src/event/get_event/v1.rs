//! [GET /_matrix/federation/v1/event/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-event-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, ServerNameBox};
use std::time::SystemTime;

ruma_api! {
    metadata: {
        description: "Retrieves a single event.",
        method: GET,
        name: "get_event",
        path: "/_matrix/federation/v1/event/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {
        /// The event ID to get.
        #[ruma_api(path)]
        pub event_id: &'a EventId,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: ServerNameBox,

        /// Time on originating homeserver when this transaction started.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        /// The event.
        #[serde(rename = "pdus", with = "ruma_serde::single_element_seq")]
        pub pdu: Pdu,
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
    pub fn new(origin: ServerNameBox, origin_server_ts: SystemTime, pdu: Pdu) -> Self {
        Self { origin, origin_server_ts, pdu }
    }
}
