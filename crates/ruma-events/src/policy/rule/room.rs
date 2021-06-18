//! Types for the *m.policy.rule.room* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{policy::rule::PolicyRuleEventContent, StateEvent};

/// This event type is used to apply rules to room entities.
pub type RoomEvent = StateEvent<RoomEventContent>;

/// The payload for `RoomEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.room", kind = State)]
pub struct RoomEventContent(pub PolicyRuleEventContent);

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use js_int::int;
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use ruma_identifiers::{event_id, room_id, user_id};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RoomEvent, RoomEventContent};
    use crate::{
        policy::rule::{PolicyRuleEventContent, Recommendation},
        Unsigned,
    };

    #[test]
    fn serialization() {
        let room_event = RoomEvent {
            event_id: event_id!("$143273582443PhrSn:example.org"),
            sender: user_id!("@example:example.org"),
            origin_server_ts: MilliSecondsSinceUnixEpoch(1_432_735_824_653_u64.try_into().unwrap()),
            room_id: room_id!("!jEsUZKDJdhlrceRyVU:example.org"),
            state_key: "rule:#*:example.org".into(),
            prev_content: None,
            unsigned: Unsigned {
                age: Some(int!(1234)),
                transaction_id: None,
                #[cfg(feature = "unstable-pre-spec")]
                relations: None,
            },
            content: RoomEventContent(PolicyRuleEventContent {
                entity: "#*:example.org".into(),
                reason: "undesirable content".into(),
                recommendation: Recommendation::Ban,
            }),
        };

        let json = json!({
            "content": {
                "entity": "#*:example.org",
                "reason": "undesirable content",
                "recommendation": "m.ban"
            },
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 1_432_735_824_653_u64,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "rule:#*:example.org",
            "type": "m.policy.rule.room",
            "unsigned": {
                "age": 1234
            }
        });

        assert_eq!(to_json_value(room_event).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "entity": "#*:example.org",
                "reason": "undesirable content",
                "recommendation": "m.ban"
            },
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 1_432_735_824_653_u64,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "rule:#*:example.org",
            "type": "m.policy.rule.room",
            "unsigned": {
                "age": 1234
            }
        });

        assert!(from_json_value::<Raw<RoomEvent>>(json).unwrap().deserialize().is_ok());
    }
}
