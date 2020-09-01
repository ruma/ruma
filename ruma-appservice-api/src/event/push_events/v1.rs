//! [PUT /_matrix/app/v1/transactions/{txnId}](https://matrix.org/docs/spec/application_service/r0.1.2#put-matrix-app-v1-transactions-txnid)

use ruma_api::ruma_api;
use ruma_common::Raw;
use ruma_events::AnyEvent;

ruma_api! {
    metadata: {
        description: "This API is called by the homeserver when it wants to push an event (or batch of events) to the application service.",
        method: PUT,
        name: "push_events",
        path: "/_matrix/app/v1/transactions/:txn_id",
        rate_limited: false,
        requires_authentication: true,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {
        /// The transaction ID for this set of events.
        ///
        /// Homeservers generate these IDs and they are used to ensure idempotency of results.
        #[ruma_api(path)]
        pub txn_id: &'a str,

        /// A list of events.
        #[ruma_api(body)]
        pub events: &'a [Raw<AnyEvent>],
    }

    #[derive(Default)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given transaction ID and list of events.
    pub fn new(txn_id: &'a str, events: &'a [Raw<AnyEvent>]) -> Self {
        Self { txn_id, events }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
