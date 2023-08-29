//! Types for the [`m.policy.rule.room`] event.
//!
//! [`m.policy.rule.room`]: https://spec.matrix.org/latest/client-server-api/#mpolicyruleroom

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::{EventContent, PossiblyRedactedStateEventContent, StateEventType};

/// The content of an `m.policy.rule.room` event.
///
/// This event type is used to apply rules to room entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.room", kind = State, state_key_type = String, custom_possibly_redacted)]
pub struct PolicyRuleRoomEventContent(pub PolicyRuleEventContent);

/// The possibly redacted form of [`PolicyRuleRoomEventContent`].
///
/// This type is used when it's not obvious whether the content is redacted or not.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct PossiblyRedactedPolicyRuleRoomEventContent(pub PossiblyRedactedPolicyRuleEventContent);

impl EventContent for PossiblyRedactedPolicyRuleRoomEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::PolicyRuleRoom
    }
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedPolicyRuleRoomEventContent {
    type StateKey = String;
}

#[cfg(test)]
mod tests {
    use ruma_common::serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{OriginalPolicyRuleRoomEvent, PolicyRuleRoomEventContent};
    use crate::policy::rule::{PolicyRuleEventContent, Recommendation};

    #[test]
    fn serialization() {
        let content = PolicyRuleRoomEventContent(PolicyRuleEventContent {
            entity: "#*:example.org".into(),
            reason: "undesirable content".into(),
            recommendation: Recommendation::Ban,
        });

        let json = json!({
            "entity": "#*:example.org",
            "reason": "undesirable content",
            "recommendation": "m.ban"
        });

        assert_eq!(to_json_value(content).unwrap(), json);
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

        from_json_value::<Raw<OriginalPolicyRuleRoomEvent>>(json).unwrap().deserialize().unwrap();
    }
}
