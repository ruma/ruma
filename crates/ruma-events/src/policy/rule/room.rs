//! Types for the [`m.policy.rule.room`] event.
//!
//! [`m.policy.rule.room`]: https://spec.matrix.org/latest/client-server-api/#mpolicyruleroom

use ruma_common::room_version_rules::RedactionRules;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::{PossiblyRedactedStateEventContent, RedactContent, StateEventType, StaticEventContent};

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

impl PossiblyRedactedStateEventContent for PossiblyRedactedPolicyRuleRoomEventContent {
    type StateKey = String;

    fn event_type(&self) -> StateEventType {
        StateEventType::PolicyRuleRoom
    }
}

impl StaticEventContent for PossiblyRedactedPolicyRuleRoomEventContent {
    const TYPE: &'static str = PolicyRuleRoomEventContent::TYPE;
    type IsPrefix = <PolicyRuleRoomEventContent as StaticEventContent>::IsPrefix;
}

impl RedactContent for PossiblyRedactedPolicyRuleRoomEventContent {
    type Redacted = Self;

    fn redact(self, _rules: &RedactionRules) -> Self::Redacted {
        Self(PossiblyRedactedPolicyRuleEventContent::empty())
    }
}

impl From<PolicyRuleRoomEventContent> for PossiblyRedactedPolicyRuleRoomEventContent {
    fn from(value: PolicyRuleRoomEventContent) -> Self {
        let PolicyRuleRoomEventContent(policy) = value;
        Self(policy.into())
    }
}

impl From<RedactedPolicyRuleRoomEventContent> for PossiblyRedactedPolicyRuleRoomEventContent {
    fn from(value: RedactedPolicyRuleRoomEventContent) -> Self {
        let RedactedPolicyRuleRoomEventContent {} = value;
        Self(PossiblyRedactedPolicyRuleEventContent::empty())
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, serde::Raw};
    use serde_json::{from_value as from_json_value, json};

    use super::{OriginalPolicyRuleRoomEvent, PolicyRuleRoomEventContent};
    use crate::policy::rule::{PolicyRuleEventContent, Recommendation};

    #[test]
    fn serialization() {
        let content = PolicyRuleRoomEventContent(PolicyRuleEventContent {
            entity: "#*:example.org".into(),
            reason: "undesirable content".into(),
            recommendation: Recommendation::Ban,
        });

        assert_to_canonical_json_eq!(
            content,
            json!({
                "entity": "#*:example.org",
                "reason": "undesirable content",
                "recommendation": "m.ban",
            }),
        );
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
