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
}

impl<'de> Deserialize<'de> for RawStrippedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

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
