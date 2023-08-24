//! Types for the [`m.key.verification.ready`] event.
//!
//! [`m.key.verification.ready`]: https://spec.matrix.org/latest/client-server-api/#mkeyverificationready

use ruma_common::{OwnedDeviceId, OwnedTransactionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::VerificationMethod;
use crate::relation::Reference;

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
    pub relates_to: Reference,
}

impl KeyVerificationReadyEventContent {
    /// Creates a new `KeyVerificationReadyEventContent` with the given device ID, methods and
    /// reference.
    pub fn new(
        from_device: OwnedDeviceId,
        methods: Vec<VerificationMethod>,
        relates_to: Reference,
    ) -> Self {
        Self { from_device, methods, relates_to }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{owned_event_id, OwnedDeviceId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{KeyVerificationReadyEventContent, ToDeviceKeyVerificationReadyEventContent};
    use crate::{key::verification::VerificationMethod, relation::Reference};

    #[test]
    fn serialization() {
        let event_id = owned_event_id!("$1598361704261elfgc:localhost");
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
            relates_to: Reference { event_id },
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
        let json_data = json!({
            "from_device": "123",
            "methods": ["m.sas.v1"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$1598361704261elfgc:localhost",
            }
        });

        let content = from_json_value::<KeyVerificationReadyEventContent>(json_data).unwrap();
        assert_eq!(content.from_device, "123");
        assert_eq!(content.methods, vec![VerificationMethod::SasV1]);
        assert_eq!(content.relates_to.event_id, "$1598361704261elfgc:localhost");

        let json_data = json!({
            "from_device": "123",
            "methods": ["m.sas.v1"],
            "transaction_id": "456",
        });

        let content =
            from_json_value::<ToDeviceKeyVerificationReadyEventContent>(json_data).unwrap();
        assert_eq!(content.from_device, "123");
        assert_eq!(content.methods, vec![VerificationMethod::SasV1]);
        assert_eq!(content.transaction_id, "456");
    }
}
