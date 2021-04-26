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

impl IncomingRequest {
    /// Creates an `IncomingRequest` with the given transaction ID and list of events.
    pub fn new(txn_id: String, events: Vec<Raw<AnyEvent>>) -> IncomingRequest {
        IncomingRequest { txn_id, events }
    }

    /// Consumes the `IncomingRequest` and tries to convert it to a `sync_events::Response`
    ///
    /// This is a helper conversion in cases where it's easier to work with `sync_events::Response`
    /// instead of the original `push_events::IncomingRequest`. It puts all events with a `room_id`
    /// into the `JoinedRoom`'s `timeline`. The rationale behind that is that incoming Appservice
    /// transactions from the homeserver are not necessarily bound to a specific user but can cover
    /// a multitude of namespaces, and as such the Appservice basically only "observes joined
    /// rooms".
    ///
    /// Note: Currently homeservers only push PDUs to appservices, no EDUs. There's the open
    /// [MSC2409] regarding supporting EDUs in the future, though it seems to be planned to put
    /// EDUs into a different JSON key than `events` to stay backwards compatible.
    ///
    /// [MSC2409]: https://github.com/matrix-org/matrix-doc/pull/2409
    #[cfg(feature = "helper")]
    pub fn try_into_sync_response(
        self,
        next_batch: impl Into<String>,
    ) -> Result<ruma_client_api::r0::sync::sync_events::Response, serde_json::Error> {
        use ruma_client_api::r0::sync::sync_events;
        use ruma_identifiers::RoomId;
        use serde::Deserialize;
        use tracing::warn;

        #[derive(Debug, Deserialize)]
        struct EventDeHelper {
            room_id: Option<RoomId>,
        }

        let mut response = sync_events::Response::new(next_batch.into());

        for raw_event in self.events {
            let helper: EventDeHelper = serde_json::from_str(raw_event.json().get())?;
            let event_json = Raw::into_json(raw_event);

            if let Some(room_id) = helper.room_id {
                let join = response.rooms.join.entry(room_id).or_default();
                join.timeline.events.push(Raw::from_json(event_json));
            } else {
                warn!("Event without room_id: {}", event_json);
            }
        }

        Ok(response)
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "helper")]
#[cfg(test)]
mod helper_tests {
    use super::{AnyEvent, IncomingRequest, Raw};
    use ruma_client_api::r0::sync::sync_events;
    use ruma_identifiers::room_id;
    use serde_json::json;

    #[test]
    fn convert_incoming_request_to_sync_response() {
        let txn_id = "any_txn_id".to_owned();
        let state_event: AnyEvent = serde_json::from_value(json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        }))
        .unwrap();
        let room_event: AnyEvent = serde_json::from_value(json!({
            "type": "m.room.message",
            "event_id": "$143273582443PhrSn:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@user:example.com",
            "content": {
                "body": "test",
                "msgtype": "m.audio",
                "url": "mxc://example.com/AuDi0",
            }
        }))
        .unwrap();

        let events = vec![Raw::from(state_event), Raw::from(room_event)];
        let incoming_request = IncomingRequest { txn_id: txn_id.clone(), events };

        let response: sync_events::Response =
            incoming_request.try_into_sync_response(txn_id).unwrap();

        let response_rooms_join =
            response.rooms.join.get(&room_id!("!roomid:room.com")).expect("joined room response");

        assert_eq!(response_rooms_join.timeline.events.len(), 2);
    }
}

#[cfg(feature = "server")]
#[cfg(test)]
mod tests {
    use ruma_api::{exports::http, OutgoingRequest, SendAccessToken};
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
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
            )
            .unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&req.body()).unwrap();

        assert_eq!(
            1,
            json_body.as_object().unwrap().get("events").unwrap().as_array().unwrap().len()
        );
    }
}
