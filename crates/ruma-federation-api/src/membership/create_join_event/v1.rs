//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1send_joinroomideventid

use ruma_common::{api::ruma_api, EventId, RoomId};
use serde_json::value::RawValue as RawJsonValue;

use super::RoomState;

ruma_api! {
    metadata: {
        description: "Send a join event to a resident server.",
        name: "create_join_event",
        method: PUT,
        stable_path: "/_matrix/federation/v1/send_join/:room_id/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
        added: 1.0,
    }

    request: {
        /// The room ID that is about to be joined.
        ///
        /// Do not use this. Instead, use the `room_id` field inside the PDU.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event ID for the join event.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The PDU.
        #[ruma_api(body)]
        pub pdu: &'a RawJsonValue,
    }

    response: {
        /// Full state and auth chain of the room prior to the join event.
        #[ruma_api(body)]
        #[serde(with = "crate::serde::v1_pdu")]
        pub room_state: RoomState,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` from the given room ID, event ID and PDU.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId, pdu: &'a RawJsonValue) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Response {
    /// Creates a new `Response` with the given room state.
    pub fn new(room_state: RoomState) -> Self {
        Self { room_state }
    }
}

#[cfg(all(test, feature = "server", not(feature = "unstable-pre-spec")))]
mod tests {
    use ruma_common::api::OutgoingResponse;
    use serde_json::{from_slice as from_json_slice, json, Value as JsonValue};

    use super::{super::RoomState, Response};

    #[test]
    fn response_body() {
        let res = Response::new(RoomState::new("ORIGIN".to_owned()))
            .try_into_http_response::<Vec<u8>>()
            .unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(res.body()).unwrap(),
            json!([200, { "auth_chain": [], "origin": "ORIGIN", "state": [] }])
        );
    }
}
