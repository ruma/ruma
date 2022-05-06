//! `PUT /_matrix/federation/*/send/{txnId}`
//!
//! Endpoint to send live activity messages to another server.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1sendtxnid

    use std::collections::BTreeMap;

    use ruma_common::{
        api::ruma_api, serde::Raw, MilliSecondsSinceUnixEpoch, OwnedEventId, ServerName,
        TransactionId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    use crate::transactions::edu::Edu;

    ruma_api! {
        metadata: {
            description: "Send transaction messages to another server",
            name: "send_transaction_message",
            method: PUT,
            stable_path: "/_matrix/federation/v1/send/:transaction_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
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
            ///
            /// With the `unstable-pre-spec` feature, sending `pdus` is optional.
            /// See [matrix-spec#705](https://github.com/matrix-org/matrix-spec/issues/705).
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
            ///
            /// With the `unstable-msc3618` feature, returning `pdus` is optional.
            /// See [MSC3618](https://github.com/matrix-org/matrix-spec-proposals/pull/3618).
            #[cfg_attr(feature = "unstable-msc3618", serde(default))]
            #[serde(with = "crate::serde::pdu_process_response")]
            pub pdus: BTreeMap<OwnedEventId, Result<(), String>>,
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
        pub fn new(pdus: BTreeMap<OwnedEventId, Result<(), String>>) -> Self {
            Self { pdus }
        }
    }
}
