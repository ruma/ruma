//! Types for the [`m.key.verification.ready`] event.
//!
//! [`m.key.verification.ready`]: https://spec.matrix.org/v1.2/client-server-api/#mkeyverificationready

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{Relation, VerificationMethod};
use crate::{OwnedDeviceId, OwnedTransactionId};

/// The content of a to-device `m.m.key.verification.ready` event.
///
/// Response to a previously sent `m.key.verification.request` message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.ready", kind = ToDevice)]
pub struct ToDeviceKeyVerificationReadyEventContent {
    /// The device ID which is initiating the request.
    pub from_device: OwnedDeviceId,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// An opaque identifier for the verification process.
    ///
    /// Must be unique with respect to the devices involved. Must be the same as the
    /// `transaction_id` given in the `m.key.verification.request` from a
    /// request.
    pub transaction_id: OwnedTransactionId,
}

impl ToDeviceKeyVerificationReadyEventContent {
    /// Creates a new `ToDeviceKeyVerificationReadyEventContent` with the given device ID,
    /// verification methods and transaction ID.
    pub fn new(
        from_device: OwnedDeviceId,
        methods: Vec<VerificationMethod>,
        transaction_id: OwnedTransactionId,
    ) -> Self {
        Self { from_device, methods, transaction_id }
    }
}

/// The content of an in-room `m.m.key.verification.ready` event.
///
/// Response to a previously sent `m.key.verification.request` message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.ready", kind = MessageLike)]
pub struct KeyVerificationReadyEventContent {
    /// The device ID which is initiating the request.
    pub from_device: OwnedDeviceId,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// Relation signaling which verification request this event is responding
    /// to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

impl KeyVerificationReadyEventContent {
    /// Creates a new `KeyVerificationReadyEventContent` with the given device ID, methods and
    /// relation.
    pub fn new(
        from_device: OwnedDeviceId,
        methods: Vec<VerificationMethod>,
        relates_to: Relation,
    ) -> Self {
        Self { from_device, methods, relates_to }
    }
}

#[cfg(test)]
mod tests {
    use crate::{event_id, OwnedDeviceId};
    use assert_matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{KeyVerificationReadyEventContent, ToDeviceKeyVerificationReadyEventContent};
    use crate::events::key::verification::{Relation, VerificationMethod};

    #[test]
    fn serialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost").to_owned();
        let device: OwnedDeviceId = "123".into();

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        let content = KeyVerificationReadyEventContent {
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

        let content = ToDeviceKeyVerificationReadyEventContent {
            from_device: device,
            transaction_id: "456".into(),
            methods: vec![VerificationMethod::SasV1],
        };

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let id = event_id!("$1598361704261elfgc:localhost");
        let device: OwnedDeviceId = "123".into();

        let json_data = json!({
            "from_device": device,
            "methods": ["m.sas.v1"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": id,
            }
        });

        assert_matches!(
            from_json_value::<KeyVerificationReadyEventContent>(json_data).unwrap(),
            KeyVerificationReadyEventContent {
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
            from_json_value::<ToDeviceKeyVerificationReadyEventContent>(json_data).unwrap(),
            ToDeviceKeyVerificationReadyEventContent {
                from_device,
                transaction_id,
                methods,
            } if from_device == device
                && methods == vec![VerificationMethod::SasV1]
                && transaction_id == "456"
        );
    }
}
