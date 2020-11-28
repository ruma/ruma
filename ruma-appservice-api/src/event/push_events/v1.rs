//! [PUT /_matrix/app/v1/transactions/{txnId}](https://matrix.org/docs/spec/application_service/r0.1.2#put-matrix-app-v1-transactions-txnid)

use ruma_api::ruma_api;
use ruma_events::AnyEvent;
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "This API is called by the homeserver when it wants to push an event (or batch of events) to the application service.",
        method: PUT,
        name: "push_events",
        path: "/_matrix/app/v1/transactions/:txn_id",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The transaction ID for this set of events.
        ///
        /// Homeservers generate these IDs and they are used to ensure idempotency of results.
        #[ruma_api(path)]
        pub txn_id: &'a str,

        /// A list of events.
        pub events: &'a [Raw<AnyEvent>],
    }

    #[derive(Default)]
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

#[cfg(test)]
mod tests {
    use ruma_api::{exports::http, OutgoingRequest};
    use ruma_events::AnyEvent;
    use ruma_serde::Raw;
    use serde_json::json;

    use super::Request;

    #[test]
    fn decode_request_contains_events_field() {
        let dummy_event: AnyEvent = serde_json::from_value(json!({
            "content": {},
            "type": "m.dummy"
        }))
        .unwrap();
        let dummy_event = Raw::from(dummy_event);
        let events = vec![dummy_event];

        let req: http::Request<Vec<u8>> = Request { events: &events, txn_id: "any_txn_id" }
            .try_into_http_request("https://homeserver.tld", Some("auth_tok"))
            .unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&req.body()).unwrap();

        assert_eq!(
            1,
            json_body.as_object().unwrap().get("events").unwrap().as_array().unwrap().len()
        );
    }
}
