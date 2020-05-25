//! [PUT /_matrix/federation/v1/send/{txnId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-txnid)

use std::{collections::BTreeMap, time::SystemTime};

use ruma_api::ruma_api;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::RoomV3Pdu;

ruma_api! {
    metadata {
        description: "Send transaction messages to another server",
        name: "send_transaction_message",
        method: PUT,
        path: "/_matrix/federation/v1/send/:transaction_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// A transaction ID unique between sending and receiving homeservers.
        #[ruma_api(path)]
        pub transaction_id: String,
        /// The server_name of the homeserver sending this transaction.
        pub origin: String,
        /// POSIX timestamp in milliseconds on originating homeserver when this transaction started.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,
        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        pub pdus: Vec<RoomV3Pdu>,
        /// List of ephemeral messages.
        ///
        /// Must not be more than 100 items.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub edus: Vec<Edu>,
    }

    response {
        /// Map of event IDs and response for each PDU given in the request.
        pub pdus: BTreeMap<EventId, PduProcessResponse>,
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

/// Error messages (if any) for a given PDU proccessing result.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PduProcessResponse {
    /// PDU processing error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
