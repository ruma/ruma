use ruma::{
    events::{
        from_raw_json_value, room::member::MembershipState, AnyStateEvent, AnyStrippedStateEvent,
        AnySyncStateEvent, EventDeHelper, EventType,
    },
    identifiers::{EventId, RoomId},
};
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;
use std::{convert::TryFrom, time::SystemTime};

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum StateEvent {
    Full(AnyStateEvent),
    Sync(AnySyncStateEvent),
    Stripped(AnyStrippedStateEvent),
}

impl StateEvent {
    pub fn is_power_event(&self) -> bool {
        match self {
            Self::Full(any_event) => match any_event {
                AnyStateEvent::RoomPowerLevels(event) => event.state_key == "",
                AnyStateEvent::RoomJoinRules(event) => event.state_key == "",
                AnyStateEvent::RoomCreate(event) => event.state_key == "",
                AnyStateEvent::RoomMember(event) => {
                    if [MembershipState::Leave, MembershipState::Ban]
                        .contains(&event.content.membership)
                    {
                        return event.sender.as_str() != event.state_key;
                    }
                    false
                }
                _ => false,
            },
            Self::Sync(any_event) => match any_event {
                AnySyncStateEvent::RoomPowerLevels(event) => event.state_key == "",
                AnySyncStateEvent::RoomJoinRules(event) => event.state_key == "",
                AnySyncStateEvent::RoomCreate(event) => event.state_key == "",
                AnySyncStateEvent::RoomMember(event) => {
                    if [MembershipState::Leave, MembershipState::Ban]
                        .contains(&event.content.membership)
                    {
                        return event.sender.as_str() != event.state_key;
                    }
                    false
                }
                _ => false,
            },
            Self::Stripped(any_event) => match any_event {
                AnyStrippedStateEvent::RoomPowerLevels(event) => event.state_key == "",
                AnyStrippedStateEvent::RoomJoinRules(event) => event.state_key == "",
                AnyStrippedStateEvent::RoomCreate(event) => event.state_key == "",
                AnyStrippedStateEvent::RoomMember(event) => {
                    if [MembershipState::Leave, MembershipState::Ban]
                        .contains(&event.content.membership)
                    {
                        return event.sender.as_str() != event.state_key;
                    }
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }
    pub fn origin_server_ts(&self) -> Option<&SystemTime> {
        match self {
            Self::Full(ev) => Some(ev.origin_server_ts()),
            Self::Sync(ev) => Some(ev.origin_server_ts()),
            Self::Stripped(ev) => None,
        }
    }
    pub fn event_id(&self) -> Option<&EventId> {
        match self {
            Self::Full(ev) => Some(ev.event_id()),
            Self::Sync(ev) => Some(ev.event_id()),
            Self::Stripped(ev) => None,
        }
    }

    pub fn is_type_and_key(&self, ev_type: EventType, state_key: &str) -> bool {
        true
    }
}

impl<'de> de::Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper {
            state_key,
            event_id,
            room_id,
            unsigned,
            ..
        } = from_raw_json_value(&json)?;

        // Determine whether the event is a full, sync, or stripped
        // based on the fields present.
        if room_id.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Full(from_raw_json_value(&json)?),
            })
        } else if event_id.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Sync(from_raw_json_value(&json)?),
            })
        } else {
            Ok(StateEvent::Stripped(from_raw_json_value(&json)?))
        }
    }
}
