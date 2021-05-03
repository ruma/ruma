//! Types for the *m.key.verification.ready* event.

use ruma_events_macros::MessageEventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};

use super::{Relation, VerificationMethod};
use crate::MessageEvent;

/// Response to a previously sent *m.key.verification.request* message.
pub type ReadyEvent = MessageEvent<ReadyEventContent>;

/// The payload for `ReadyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.key.verification.ready")]
pub struct ReadyEventContent {
    /// The device ID which is initiating the request.
    pub from_device: DeviceIdBox,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// Relation signaling which verification request this event is responding
    /// to.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::{event_id, DeviceIdBox};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{ReadyEventContent, Relation, VerificationMethod};

    #[test]
    fn serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");
        let device: DeviceIdBox = "123".into();

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        let content = ReadyEventContent {
            from_device: device,
            relation: Relation { event_id },
            methods: vec![VerificationMethod::MSasV1],
        };

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");
        let device: DeviceIdBox = "123".into();

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        assert_matches!(
            from_json_value::<Raw<ReadyEventContent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            ReadyEventContent {
                from_device,
                relation: Relation {
                    event_id
                },
                methods,
            } if from_device == device
                && methods == vec![VerificationMethod::MSasV1]
                && event_id == id
        );
    }
}
