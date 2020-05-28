//! [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-join-roomid-eventid)

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};

use super::RoomState;
use crate::pdu::PduStub;

ruma_api! {
    metadata {
        description: "Send a join event to a resident server.",
        name: "create_join_event",
        method: PUT,
        path: "/_matrix/federation/v1/send_join/:room_id/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub event_id: EventId,

        /// PDU type without event and room IDs.
        #[serde(flatten)]
        pub pdu_stub: PduStub,
    }

    response {
        /// Full state of the room.
        #[ruma_api(body)]
        #[serde(with = "crate::serde::room_state")]
        pub room_state: RoomState,
    }
}

/*
#[cfg(test)]
mod tests {

    use std::{collections::BTreeMap, convert::TryFrom, time::SystemTime};

    use ruma_events::{room::create::CreateEventContent, EventJson, EventType};
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::json;

    use super::Request;
    use crate::pdu::EventHash;
    use crate::pdu::{PduStub, RoomV1PduStub};

    #[test]
    fn test_serialize_request() {
        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);
        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));
        let request = Request {
            room_id: RoomId::try_from("someroomid").unwrap(),
            event_id: EventId::try_from("12345").unwrap(),
            pdu_stub: PduStub::RoomV1PduStub(RoomV1PduStub {
                sender: UserId::try_from("@sender:example.com").unwrap(),
                origin: "matrix.org".to_string(),
                origin_server_ts: SystemTime::now(),
                kind: EventType::RoomPowerLevels,
                content: json!({"testing": 123}),
                state_key: Some("state".to_string()),
                prev_events: vec![(
                    EventId::try_from("!previousevent:matrix.org").unwrap(),
                    EventHash {
                        sha256: "123567".to_string(),
                    },
                )],
                depth: 2_u32.into(),
                auth_events: vec![(
                    EventId::try_from("!someauthevent:matrix.org").unwrap(),
                    EventHash {
                        sha256: "21389CFEDABC".to_string(),
                    },
                )],
                redacts: Some(EventId::try_from("9654").unwrap()),
                unsigned,
                hashes: EventHash {
                    sha256: "1233543bABACDEF".to_string(),
                },
                signatures,
            }),
        };

        TryFrom::<http::Request<Vec<u8>>>(request);
        println!("{}", serde_json::to_string_pretty(&request).unwrap());
    }
}
*/
