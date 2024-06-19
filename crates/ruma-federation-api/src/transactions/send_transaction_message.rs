//! `PUT /_matrix/federation/*/send/{txnId}`
//!
//! Send live activity messages to another server.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1sendtxnid

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedServerName, OwnedTransactionId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    use crate::transactions::edu::Edu;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/send/{transaction_id}",
        }
    };

    /// Request type for the `send_transaction_message` endpoint.
    #[request]
    pub struct Request {
        /// A transaction ID unique between sending and receiving homeservers.
        #[ruma_api(path)]
        pub transaction_id: OwnedTransactionId,

        /// The server_name of the homeserver sending this transaction.
        pub origin: OwnedServerName,

        /// POSIX timestamp in milliseconds on the originating homeserver when this transaction
        /// started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        ///
        /// With the `unstable-unspecified` feature, sending `pdus` is optional.
        /// See [matrix-spec#705](https://github.com/matrix-org/matrix-spec/issues/705).
        #[cfg_attr(
            feature = "unstable-unspecified",
            serde(default, skip_serializing_if = "<[_]>::is_empty")
        )]
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
        pub pdus: BTreeMap<OwnedEventId, Result<(), String>>,
    }

    impl Request {
        /// Creates a new `Request` with the given transaction ID, origin, timestamp.
        ///
        /// The PDU and EDU lists will start off empty.
        pub fn new(
            transaction_id: OwnedTransactionId,
            origin: OwnedServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
        ) -> Self {
            Self { transaction_id, origin, origin_server_ts, pdus: vec![], edus: vec![] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given PDUs.
        pub fn new(pdus: BTreeMap<OwnedEventId, Result<(), String>>) -> Self {
            Self { pdus }
        }
    }
}
