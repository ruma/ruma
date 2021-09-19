//! Types for the *m.key.verification.ready* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};

use super::{Relation, VerificationMethod};
use crate::MessageEvent;

/// Response to a previously sent *m.key.verification.request* message.
pub type ReadyEvent = MessageEvent<ReadyEventContent>;

/// The payload for a to-device `m.key.verification.ready` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.ready", kind = ToDevice)]
pub struct ReadyToDeviceEventContent {
    /// The device ID which is initiating the request.
    pub from_device: DeviceIdBox,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the *m.key.verification.request* from a
    /// request.
    pub transaction_id: String,
}

impl ReadyToDeviceEventContent {
    /// Creates a new `ReadyToDeviceEventContent` with the given device ID, verification methods and
    /// transaction ID.
    pub fn new(
        from_device: DeviceIdBox,
        methods: Vec<VerificationMethod>,
        transaction_id: String,
    ) -> Self {
        Self { from_device, methods, transaction_id }
    }
}

/// The payload for an in-room `m.key.verification.ready` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.ready", kind = Message)]
pub struct ReadyEventContent {
    /// The device ID which is initiating the request.
    pub from_device: DeviceIdBox,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// Relation signaling which verification request this event is responding
    /// to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

impl ReadyEventContent {
    /// Creates a new `ReadyEventContent` with the given device ID, methods and relation.
    pub fn new(
        from_device: DeviceIdBox,
        methods: Vec<VerificationMethod>,
        relates_to: Relation,
    ) -> Self {
        Self { from_device, methods, relates_to }
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::{event_id, DeviceIdBox};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{ReadyEventContent, ReadyToDeviceEventContent};
    use crate::key::verification::{Relation, VerificationMethod};

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
            from_device: device.clone(),
            relates_to: Relation { event_id },
            methods: vec![VerificationMethod::SasV1],
        };

        assert_eq!(to_json_value(&content).unwrap(), json_data);

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "transaction_id": "456",
        });

        let content = ReadyToDeviceEventContent {
            from_device: device,
            transaction_id: "456".to_owned(),
            methods: vec![VerificationMethod::SasV1],
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
            from_json_value::<ReadyEventContent>(json_data).unwrap(),
            ReadyEventContent {
                from_device,
                relates_to: Relation {
                    event_id
                },
                methods,
            } if from_device == device
                && methods == vec![VerificationMethod::SasV1]
                && event_id == id
        );

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "transaction_id": "456",
        });

        assert_matches!(
            from_json_value::<ReadyToDeviceEventContent>(json_data).unwrap(),
            ReadyToDeviceEventContent {
                from_device,
                transaction_id,
                methods,
            } if from_device == device
                && methods == vec![VerificationMethod::SasV1]
                && transaction_id == "456"
        );
    }
}
