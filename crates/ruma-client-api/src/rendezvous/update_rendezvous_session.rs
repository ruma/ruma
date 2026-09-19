//! `PUT /_matrix/client/*/rendezvous/{id}/{txnId}`
//!
//! Update a rendezvous session.

pub mod unstable {
    //! `unstable/io.element.msc4388` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4388

    use ruma_common::{
        TransactionId,
        api::{auth_scheme::NoAccessToken, request, response},
        metadata,
    };

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: NoAccessToken,
        history: {
            unstable => "/_matrix/client/unstable/io.element.msc4388/rendezvous/{id}/{txn_id}",
        }
    }

    /// Request type for the `PUT` `rendezvous` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the rendezvous session to update.
        #[ruma_api(path)]
        pub id: String,

        /// The transaction ID for this update.
        ///
        /// It is used by the server to ensure idempotency of requests: if the server has already
        /// seen this transaction ID for this rendezvous session, it returns the recorded response
        /// instead of evaluating the request again. A client that changes the `sequence_token` or
        /// `data` it sends must use a new transaction ID.
        #[ruma_api(path)]
        pub txn_id: TransactionId,

        /// The expected sequence token for the session. If it doesn't match the server state then
        /// an error is returned.
        pub sequence_token: String,

        /// Data up to maximum size allowed by the server.
        pub data: String,
    }

    impl Request {
        /// Creates a new `Request` with the given id, transaction ID, sequence token and data.
        pub fn new(
            id: String,
            txn_id: TransactionId,
            sequence_token: String,
            data: String,
        ) -> Self {
            Self { id, txn_id, sequence_token, data }
        }
    }

    /// Response type for the `PUT` `rendezvous` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The new sequence token for the session.
        pub sequence_token: String,
    }

    impl Response {
        /// Creates a new `Response` with the given sequence token.
        pub fn new(sequence_token: String) -> Self {
            Self { sequence_token }
        }
    }
}
