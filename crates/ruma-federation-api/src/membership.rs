//! Room membership endpoints.

use ruma_common::serde::Raw;
use ruma_events::AnyStrippedStateEvent;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

pub mod create_invite;
pub mod create_join_event;
pub mod create_knock_event;
pub mod create_leave_event;
pub mod prepare_join_event;
pub mod prepare_knock_event;
pub mod prepare_leave_event;

/// Possible event formats that may appear in stripped state.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RawStrippedState {
    /// A stripped state event.
    Stripped(Raw<AnyStrippedStateEvent>),

    /// A full federation PDU.
    #[cfg(feature = "unstable-msc4311")]
    Pdu(Box<RawJsonValue>),
}

impl<'de> Deserialize<'de> for RawStrippedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        #[cfg(feature = "unstable-msc4311")]
        {
            use ruma_common::serde::from_raw_json_value;
            use serde::de;

            #[derive(Deserialize)]
            struct PotentialPduDeHelper {
                auth_events: Option<de::IgnoredAny>,
                prev_events: Option<de::IgnoredAny>,
                signatures: Option<de::IgnoredAny>,
                hashes: Option<de::IgnoredAny>,
            }

            let PotentialPduDeHelper { auth_events, prev_events, signatures, hashes } =
                from_raw_json_value(&json)?;

            if auth_events.is_some()
                && prev_events.is_some()
                && signatures.is_some()
                && hashes.is_some()
            {
                return Ok(Self::Pdu(json));
            }
        }

        Ok(Raw::from_json(json).into())
    }
}

impl From<Raw<AnyStrippedStateEvent>> for RawStrippedState {
    fn from(value: Raw<AnyStrippedStateEvent>) -> Self {
        Self::Stripped(value)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{serde::Raw, user_id};
    use ruma_events::{room::member::MembershipState, AnyStrippedStateEvent};
    use serde_json::{from_value as from_json_value, json};

    use super::RawStrippedState;

    #[test]
    fn deserialize_stripped_state() {
        let user_id = user_id!("@patrick:localhost");
        let content = json!({
            "membership": "join",
        });

        // Stripped format.
        let stripped_event_json = json!({
            "content": content,
            "sender": user_id,
            "state_key": user_id,
            "type": "m.room.member",
        });
        assert_matches!(
            from_json_value::<RawStrippedState>(stripped_event_json).unwrap(),
            RawStrippedState::Stripped(raw_stripped_event)
        );
        assert_matches!(
            raw_stripped_event.deserialize().unwrap(),
            AnyStrippedStateEvent::RoomMember(stripped_member_event)
        );
        assert_eq!(stripped_member_event.sender, user_id);
        assert_eq!(stripped_member_event.state_key, user_id);
        assert_eq!(stripped_member_event.content.membership, MembershipState::Join);

        #[cfg(feature = "unstable-msc4311")]
        {
            // PDU format
            let pdu_event_json = json!({
                "auth_events": [
                    "$one",
                    "$two",
                    "$three"
                ],
                "content": content,
                "depth": 10,
                "hashes": {
                    "sha256": "thisisahash"
                },
                "origin_server_ts": 1_000_000,
                "prev_events": [
                    "$one",
                    "$two",
                    "$three"
                ],
                "room_id": "!room:localhost",
                "sender": user_id,
                "signatures": {
                    "localhost": {
                        "ed25519:1": "thisisakey"
                    }
                },
                "state_key": user_id,
                "type": "m.room.member",
            });
            assert_matches!(
                from_json_value::<RawStrippedState>(pdu_event_json).unwrap(),
                RawStrippedState::Pdu(_pdu_member_event)
            );
        }
    }

    #[test]
    fn serialize_stripped_state() {
        let user_id = user_id!("@patrick:localhost");
        let content = json!({
            "membership": "join",
        });

        // Stripped format.
        let stripped_event_json = json!({
            "content": content,
            "sender": user_id,
            "state_key": user_id,
            "type": "m.room.member",
        });
        let raw_stripped_event =
            Raw::new(&stripped_event_json).unwrap().cast_unchecked::<AnyStrippedStateEvent>();
        let stripped_state = RawStrippedState::from(raw_stripped_event);

        let stripped_event_json = serde_json::to_string(&stripped_state).unwrap();
        assert_eq!(
            stripped_event_json,
            r#"{"content":{"membership":"join"},"sender":"@patrick:localhost","state_key":"@patrick:localhost","type":"m.room.member"}"#
        );
    }
}
