//! Types for `m.room.encrypted` state events, as defined in [MSC4362][msc].
//!
//! [msc]: https://github.com/matrix-org/matrix-spec-proposals/pull/4362
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    PossiblyRedactedStateEventContent, StateEventType, StaticEventContent,
    room::encrypted::EncryptedEventScheme,
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

/// The possibly redacted form of [`StateRoomEncryptedEventContent`].
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
    use js_int::uint;
    use ruma_common::{MilliSecondsSinceUnixEpoch, room_id, user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::{
        AnyStateEvent, StateEvent,
        room::encrypted::{
            EncryptedEventScheme, MegolmV1AesSha2ContentInit,
            unstable_state::StateRoomEncryptedEventContent,
        },
    };

    #[test]
    fn serialize_content() {
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
    fn deserialize_content() {
        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "session_id": "session_id",
        });

        let content: StateRoomEncryptedEventContent = from_json_value(json_data).unwrap();

        assert_matches!(content.scheme, EncryptedEventScheme::MegolmV1AesSha2(scheme));
        assert_eq!(scheme.ciphertext, "ciphertext");
        assert_eq!(scheme.sender_key, None);
        assert_eq!(scheme.device_id, None);
        assert_eq!(scheme.session_id, "session_id");
    }

    #[test]
    #[allow(deprecated)]
    fn deserialize_event() {
        let json_data = json!({
            "type": "m.room.encrypted",
            "event_id": "$event_id:example.com",
            "room_id": "!roomid:example.com",
            "sender": "@example:example.com",
            "origin_server_ts": 1_234_567_890,
            "state_key": "",
            "content": {
                "algorithm": "m.megolm.v1.aes-sha2",
                "ciphertext": "ciphertext",
                "session_id": "session_id",
            }
        });
        let event = from_json_value::<AnyStateEvent>(json_data).unwrap();

        assert_matches!(event, AnyStateEvent::RoomEncrypted(StateEvent::Original(ev)));

        assert_matches!(ev.content.scheme, EncryptedEventScheme::MegolmV1AesSha2(scheme));
        assert_eq!(scheme.ciphertext, "ciphertext");
        assert_eq!(scheme.sender_key, None);
        assert_eq!(scheme.device_id, None);
        assert_eq!(scheme.session_id, "session_id");

        assert_eq!(ev.sender, user_id!("@example:example.com"));
        assert_eq!(ev.room_id, room_id!("!roomid:example.com"));
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1_234_567_890)));
        assert_eq!(ev.state_key, "");
    }
}
