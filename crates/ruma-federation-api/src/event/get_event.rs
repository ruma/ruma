//! `GET /_matrix/federation/*/event/{eventId}`
//!
//! Retrieves a single event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1eventeventid

    use ruma_common::{api::ruma_api, EventId, MilliSecondsSinceUnixEpoch, OwnedServerName};
    use serde_json::value::RawValue as RawJsonValue;

    ruma_api! {
        metadata: {
            description: "Retrieves a single event.",
            method: GET,
            name: "get_event",
            stable_path: "/_matrix/federation/v1/event/:event_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// The event ID to get.
            #[ruma_api(path)]
            pub event_id: &'a EventId,
        }

        response: {
            /// The `server_name` of the homeserver sending this transaction.
            pub origin: OwnedServerName,

            /// Time on originating homeserver when this transaction started.
            pub origin_server_ts: MilliSecondsSinceUnixEpoch,

            /// The event.
            #[serde(rename = "pdus", with = "ruma_common::serde::single_element_seq")]
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
            origin: OwnedServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
            pdu: Box<RawJsonValue>,
        ) -> Self {
            Self { origin, origin_server_ts, pdu }
        }
    }
}
