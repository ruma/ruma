//! Types for the [`m.key.verification.done`] event.
//!
//! [`m.key.verification.done`]: https://spec.matrix.org/latest/client-server-api/#mkeyverificationdone

use ruma_common::OwnedTransactionId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::relation::Reference;

/// The content of a to-device `m.m.key.verification.done` event.
///
/// Event signaling that the interactive key verification has successfully concluded.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.done", kind = ToDevice)]
pub struct ToDeviceKeyVerificationDoneEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the `m.key.verification.start` message.
    pub transaction_id: OwnedTransactionId,
}

impl ToDeviceKeyVerificationDoneEventContent {
    /// Creates a new `ToDeviceKeyVerificationDoneEventContent` with the given transaction ID.
    pub fn new(transaction_id: OwnedTransactionId) -> Self {
        Self { transaction_id }
    }
}

/// The payload for a in-room `m.key.verification.done` event.
///
/// Event signaling that the interactive key verification has successfully concluded.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.done", kind = MessageLike)]
pub struct KeyVerificationDoneEventContent {
    /// Relation signaling which verification request this event is responding to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl KeyVerificationDoneEventContent {
    /// Creates a new `KeyVerificationDoneEventContent` with the given reference.
    pub fn new(relates_to: Reference) -> Self {
        Self { relates_to }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::owned_event_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::KeyVerificationDoneEventContent;
    use crate::relation::Reference;

    #[test]
    fn serialization() {
        let event_id = owned_event_id!("$1598361704261elfgc:localhost");

        let json_data = json!({
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": event_id,
            }
        });

        let content = KeyVerificationDoneEventContent { relates_to: Reference { event_id } };

        assert_eq!(to_json_value(&content).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$1598361704261elfgc:localhost",
            }
        });

        let content = from_json_value::<KeyVerificationDoneEventContent>(json_data).unwrap();
        assert_eq!(content.relates_to.event_id, "$1598361704261elfgc:localhost");
    }
}
