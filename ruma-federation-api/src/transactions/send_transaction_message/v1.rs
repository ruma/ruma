//! [PUT /_matrix/federation/v1/send/{txnId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-txnid)

use std::{collections::BTreeMap, time::SystemTime};

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, ServerName};
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

ruma_api! {
    metadata: {
        description: "Send transaction messages to another server",
        name: "send_transaction_message",
        method: PUT,
        path: "/_matrix/federation/v1/send/:transaction_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// A transaction ID unique between sending and receiving homeservers.
        #[ruma_api(path)]
        pub transaction_id: &'a str,

        /// The server_name of the homeserver sending this transaction.
        pub origin: &'a ServerName,

        /// POSIX timestamp in milliseconds on the originating homeserver when this transaction
        /// started.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        #[cfg_attr(feature = "unstable-pre-spec", serde(default, skip_serializing_if = "<[_]>::is_empty"))]
        pub pdus: &'a [Raw<Pdu>],

        /// List of ephemeral messages.
        ///
        /// Must not be more than 100 items.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub edus: &'a [Raw<Edu>],
    }

    #[derive(Default)]
    response: {
        /// Map of event IDs and response for each PDU given in the request.
        #[serde(with = "crate::serde::pdu_process_response")]
        pub pdus: BTreeMap<EventId, Result<(), String>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given transaction ID, origin, timestamp.
    ///
    /// The PDU and EDU lists will start off empty.
    pub fn new(
        transaction_id: &'a str,
        origin: &'a ServerName,
        origin_server_ts: SystemTime,
    ) -> Self {
        Self { transaction_id, origin, origin_server_ts, pdus: &[], edus: &[] }
    }
}

impl Response {
    /// Creates a new `Response` with the given PDUs.
    pub fn new(pdus: BTreeMap<EventId, Result<(), String>>) -> Self {
        Self { pdus }
    }
}

/// Type for passing ephemeral data to homeservers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edu {
    /// Type of the ephemeral message.
    pub edu_type: String,

    /// Content of ephemeral message
    pub content: JsonValue,
}
