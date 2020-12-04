//! Types for the *m.key.verification.done* event.

use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};

use super::Relation;
use crate::MessageEvent;

/// Event signaling that the interactive key verification has successfully
/// concluded.
pub type DoneEvent = MessageEvent<DoneEventContent>;

/// The payload for `DoneEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.done")]
pub struct DoneEventContent {
    /// Relation signaling which verification request this event is responding
    /// to.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::event_id;
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DoneEventContent, Relation};

    #[test]
    fn serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let json_data = json!({
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        let content = DoneEventContent { relation: Relation { event_id } };

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");

        let json_data = json!({
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        assert_matches!(
            from_json_value::<Raw<DoneEventContent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            DoneEventContent {
                relation: Relation {
                    event_id
                },
            } if event_id == id
        );
    }
}
