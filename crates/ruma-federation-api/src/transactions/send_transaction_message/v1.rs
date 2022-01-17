//! [PUT /_matrix/federation/v1/send/{txnId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v1-send-txnid)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{EventId, ServerName, TransactionId};
use ruma_serde::Raw;
use serde_json::value::RawValue as RawJsonValue;

use crate::transactions::edu::Edu;

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
        pub transaction_id: &'a TransactionId,

        /// The server_name of the homeserver sending this transaction.
        pub origin: &'a ServerName,

        /// POSIX timestamp in milliseconds on the originating homeserver when this transaction
        /// started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        #[cfg_attr(feature = "unstable-pre-spec", serde(default, skip_serializing_if = "<[_]>::is_empty"))]
        pub pdus: &'a [Box<RawJsonValue>],

        /// List of ephemeral messages.
        ///
        /// Must not be more than 100 items.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub edus: &'a [Raw<Edu>],
    }

    #[derive(Default)]
    response: {
        /// Map of event IDs and response for each PDU given in the request.
        // https://github.com/matrix-org/matrix-doc/pull/3618 makes returning `pdus` optional.
        #[cfg_attr(feature = "unstable-pre-spec", serde(default))]
        #[serde(with = "crate::serde::pdu_process_response")]
        pub pdus: BTreeMap<Box<EventId>, Result<(), String>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given transaction ID, origin, timestamp.
    ///
    /// The PDU and EDU lists will start off empty.
    pub fn new(
        transaction_id: &'a TransactionId,
        origin: &'a ServerName,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self { transaction_id, origin, origin_server_ts, pdus: &[], edus: &[] }
    }
}

impl Response {
    /// Creates a new `Response` with the given PDUs.
    pub fn new(pdus: BTreeMap<Box<EventId>, Result<(), String>>) -> Self {
        Self { pdus }
    }
}
