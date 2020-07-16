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

    request: {
        /// The transaction ID for this set of events.
        ///
        /// Homeservers generate these IDs and they are used to ensure idempotency of results.
        #[ruma_api(path)]
        pub txn_id: String,
        /// A list of events.
        #[ruma_api(body)]
        pub events: Vec<Raw<AnyEvent>>,
    }

    response: {}
}
