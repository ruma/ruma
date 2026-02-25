//! `GET /_matrix/federation/*/event/{eventId}`
//!
//! Retrieves a single event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1eventeventid

    use ruma_common::{
        EventId, MilliSecondsSinceUnixEpoch, ServerName,
        api::{request, response},
        metadata,
    };
    use serde_json::value::RawValue as RawJsonValue;

    use crate::authentication::ServerSignatures;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/event/{event_id}",
    }

    /// Request type for the `get_event` endpoint.
    #[request]
    pub struct Request {
        /// The event ID to get.
        #[ruma_api(path)]
        pub event_id: EventId,
    }

    /// Response type for the `get_event` endpoint.
    #[response]
    pub struct Response {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: ServerName,

        /// Time on originating homeserver when this transaction started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The event.
        #[serde(rename = "pdus", with = "ruma_common::serde::single_element_seq")]
        pub pdu: Box<RawJsonValue>,
    }

    impl Request {
        /// Creates a new `Request` with the given event id.
        pub fn new(event_id: EventId) -> Self {
            Self { event_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given server name, timestamp, and event.
        pub fn new(
            origin: ServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
            pdu: Box<RawJsonValue>,
        ) -> Self {
            Self { origin, origin_server_ts, pdu }
        }
    }
}
