//! Types for `m.room.encrypted` state events, as defined in [MSC3414][msc].
//!
//! [msc]: https://github.com/matrix-org/matrix-spec-proposals/pull/3414
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    room::encrypted::EncryptedEventScheme, PossiblyRedactedStateEventContent, StateEventType,
    StaticEventContent,
};

/// The content of an `m.room.encrypted` state event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = State, state_key_type = String, custom_possibly_redacted)]
pub struct StateRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,
}

/// The PossiblyRedacted form of [StateRoomEncryptedEventContent].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PossiblyRedactedStateRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<EncryptedEventScheme>,
}

impl StaticEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    const TYPE: &'static str = StateRoomEncryptedEventContent::TYPE;
    type IsPrefix = <StateRoomEncryptedEventContent as StaticEventContent>::IsPrefix;
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    type StateKey = String;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomEncrypted
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::room::encrypted::{
        unstable_state::StateRoomEncryptedEventContent, EncryptedEventScheme,
        MegolmV1AesSha2ContentInit,
    };

    #[test]
    fn serialization() {
        let key_verification_start_content = StateRoomEncryptedEventContent {
            scheme: EncryptedEventScheme::MegolmV1AesSha2(
                MegolmV1AesSha2ContentInit {
                    ciphertext: "ciphertext".into(),
                    sender_key: "sender_key".into(),
                    device_id: "device_id".into(),
                    session_id: "session_id".into(),
                }
                .into(),
            ),
        };

        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id",
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);
    }

    #[test]
    #[allow(deprecated)]
    fn deserialization() {
        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id",
        });

        let content: StateRoomEncryptedEventContent = from_json_value(json_data).unwrap();

        assert_matches!(content.scheme, EncryptedEventScheme::MegolmV1AesSha2(scheme));
        assert_eq!(scheme.ciphertext, "ciphertext");
        assert_eq!(scheme.sender_key, "sender_key");
        assert_eq!(scheme.device_id, "device_id");
        assert_eq!(scheme.session_id, "session_id");
    }
}
