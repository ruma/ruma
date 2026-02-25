//! `PUT /_matrix/federation/*/send/{txnId}`
//!
//! Send live activity messages to another server.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1sendtxnid

    use std::collections::BTreeMap;

    use ruma_common::{
        EventId, MilliSecondsSinceUnixEpoch, ServerName, TransactionId,
        api::{request, response},
        metadata,
        serde::Raw,
    };
    use serde_json::value::RawValue as RawJsonValue;

    use crate::{authentication::ServerSignatures, transactions::edu::Edu};

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/send/{transaction_id}",
    }

    /// Request type for the `send_transaction_message` endpoint.
    #[request]
    pub struct Request {
        /// A transaction ID unique between sending and receiving homeservers.
        #[ruma_api(path)]
        pub transaction_id: TransactionId,

        /// The server_name of the homeserver sending this transaction.
        pub origin: ServerName,

        /// POSIX timestamp in milliseconds on the originating homeserver when this transaction
        /// started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        ///
        /// With the `compat-optional-pdus` feature, this field is optional in deserialization,
        /// defaulting to an empty `Vec`.
        #[cfg_attr(feature = "compat-optional-txn-pdus", serde(default))]
        pub pdus: Vec<Box<RawJsonValue>>,

        /// List of ephemeral messages.
        ///
        /// Must not be more than 100 items.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub edus: Vec<Raw<Edu>>,
    }

    /// Response type for the `send_transaction_message` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Map of event IDs and response for each PDU given in the request.
        ///
        /// With the `unstable-msc3618` feature, returning `pdus` is optional.
        /// See [MSC3618](https://github.com/matrix-org/matrix-spec-proposals/pull/3618).
        #[cfg_attr(feature = "unstable-msc3618", serde(default))]
        #[serde(with = "crate::serde::pdu_process_response")]
        pub pdus: BTreeMap<EventId, Result<(), String>>,
    }

    impl Request {
        /// Creates a new `Request` with the given transaction ID, origin, timestamp.
        ///
        /// The PDU and EDU lists will start off empty.
        pub fn new(
            transaction_id: TransactionId,
            origin: ServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
        ) -> Self {
            Self { transaction_id, origin, origin_server_ts, pdus: vec![], edus: vec![] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given PDUs.
        pub fn new(pdus: BTreeMap<EventId, Result<(), String>>) -> Self {
            Self { pdus }
        }
    }
}
