use ruma::{
    events::{
        from_raw_json_value,
        pdu::{Pdu, PduStub, RoomV1Pdu, RoomV1PduStub, RoomV3Pdu, RoomV3PduStub},
        room::member::{MemberEventContent, MembershipState},
        AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, EventDeHelper, EventType,
    },
    identifiers::{EventId, RoomId},
};
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;
use std::{convert::TryFrom, time::SystemTime};

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum StateEvent {
    Full(Pdu),
    Sync(PduStub),
}

impl StateEvent {
    pub fn is_power_event(&self) -> bool {
        match self {
            Self::Full(any_event) => match any_event {
                Pdu::RoomV1Pdu(event) => match event.kind {
                    EventType::RoomPowerLevels
                    | EventType::RoomJoinRules
                    | EventType::RoomCreate => event.state_key == Some("".into()),
                    EventType::RoomMember => {
                        if let Ok(content) =
                            // TODO fix clone
                            serde_json::from_value::<MemberEventContent>(event.content.clone())
                        {
                            if [MembershipState::Leave, MembershipState::Ban]
                                .contains(&content.membership)
                            {
                                return event.sender.as_str()
                                    // TODO does None here mean the same as state_key = ""
                                    != event.state_key.as_deref().unwrap_or("");
                            }
                        }

                        false
                    }
                    _ => false,
                },
                Pdu::RoomV3Pdu(event) => event.state_key == Some("".into()),
            },
            Self::Sync(any_event) => match any_event {
                PduStub::RoomV1PduStub(event) => match event.kind {
                    EventType::RoomPowerLevels
                    | EventType::RoomJoinRules
                    | EventType::RoomCreate => event.state_key == Some("".into()),
                    EventType::RoomMember => {
                        if let Ok(content) =
                            serde_json::from_value::<MemberEventContent>(event.content.clone())
                        {
                            if [MembershipState::Leave, MembershipState::Ban]
                                .contains(&content.membership)
                            {
                                return event.sender.as_str()
                                    // TODO does None here mean the same as state_key = ""
                                    != event.state_key.as_deref().unwrap_or("");
                            }
                        }

                        false
                    }
                    _ => false,
                },
                PduStub::RoomV3PduStub(event) => event.state_key == Some("".into()),
            },
        }
    }
    pub fn origin_server_ts(&self) -> &SystemTime {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.origin_server_ts,
                Pdu::RoomV3Pdu(ev) => &ev.origin_server_ts,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.origin_server_ts,
                PduStub::RoomV3PduStub(ev) => &ev.origin_server_ts,
            },
        }
    }
    pub fn event_id(&self) -> Option<&EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => Some(&ev.event_id),
                Pdu::RoomV3Pdu(ev) => None,
            },
            Self::Sync(ev) => None,
        }
    }

    pub fn room_id(&self) -> Option<&RoomId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => Some(&ev.room_id),
                Pdu::RoomV3Pdu(ev) => Some(&ev.room_id),
            },
            Self::Sync(ev) => None,
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
        } else {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Sync(from_raw_json_value(&json)?),
            })
        }
    }
}
