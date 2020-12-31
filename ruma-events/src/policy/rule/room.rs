//! Types for the *m.policy.rule.room* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::{policy::rule::PolicyRuleEventContent, StateEvent};

/// This event type is used to apply rules to room entities.
pub type RoomEvent = StateEvent<RoomEventContent>;

/// The payload for `RoomEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.policy.rule.room")]
pub struct RoomEventContent(pub PolicyRuleEventContent);

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

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
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1_432_735_824_653),
            room_id: room_id!("!jEsUZKDJdhlrceRyVU:example.org"),
            state_key: "rule:#*:example.org".into(),
            prev_content: None,
            unsigned: Unsigned {
                age: Some(1234.into()),
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
            "origin_server_ts": 1432735824653u64,
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
            "origin_server_ts": 1432735824653u64,
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
