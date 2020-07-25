use std::collections::BTreeMap;

use ruma::{
    events::{
        from_raw_json_value,
        pdu::{Pdu, PduStub},
        room::member::{MemberEventContent, MembershipState},
        EventDeHelper, EventType,
    },
    identifiers::{EventId, RoomId, ServerName, UserId},
};
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;
use std::time::SystemTime;

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
                                    // TODO is None here a failure
                                    != event.state_key.as_deref().unwrap_or("NOT A STATE KEY");
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
    pub fn deserialize_content<C: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<C, serde_json::Error> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => serde_json::from_value(ev.content.clone()),
                Pdu::RoomV3Pdu(ev) => serde_json::from_value(ev.content.clone()),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => serde_json::from_value(ev.content.clone()),
                PduStub::RoomV3PduStub(ev) => serde_json::from_value(ev.content.clone()),
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
                Pdu::RoomV3Pdu(_) => None,
            },
            Self::Sync(_) => None,
        }
    }

    pub fn sender(&self) -> &UserId {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.sender,
                Pdu::RoomV3Pdu(ev) => &ev.sender,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.sender,
                PduStub::RoomV3PduStub(ev) => &ev.sender,
            },
        }
    }

    pub fn redacts(&self) -> Option<&EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.redacts.as_ref(),
                PduStub::RoomV3PduStub(ev) => ev.redacts.as_ref(),
            },
        }
    }

    pub fn room_id(&self) -> Option<&RoomId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => Some(&ev.room_id),
                Pdu::RoomV3Pdu(ev) => Some(&ev.room_id),
            },
            Self::Sync(_) => None,
        }
    }
    pub fn kind(&self) -> EventType {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.kind.clone(),
                Pdu::RoomV3Pdu(ev) => ev.kind.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.kind.clone(),
                PduStub::RoomV3PduStub(ev) => ev.kind.clone(),
            },
        }
    }
    pub fn state_key(&self) -> Option<String> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.state_key.clone(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.state_key.clone(),
                PduStub::RoomV3PduStub(ev) => ev.state_key.clone(),
            },
        }
    }

    pub fn prev_event_ids(&self) -> Vec<EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.prev_events.iter().cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.prev_events.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.prev_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.prev_events.clone(),
            },
        }
    }

    pub fn auth_event_ids(&self) -> Vec<EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.auth_events.iter().cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.auth_events.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.auth_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.auth_events.clone(),
            },
        }
    }

    pub fn content(&self) -> &serde_json::Value {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.content,
                Pdu::RoomV3Pdu(ev) => &ev.content,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.content,
                PduStub::RoomV3PduStub(ev) => &ev.content,
            },
        }
    }

    pub fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<String, String>> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(_) => maplit::btreemap! {},
                Pdu::RoomV3Pdu(ev) => ev.signatures.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.signatures.clone(),
                PduStub::RoomV3PduStub(ev) => ev.signatures.clone(),
            },
        }
    }

    pub fn is_type_and_key(&self, ev_type: EventType, state_key: &str) -> bool {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
                Pdu::RoomV3Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
                PduStub::RoomV3PduStub(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
            },
        }
    }
}

impl<'de> de::Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper {
            room_id, unsigned, ..
        } = from_raw_json_value(&json)?;

        // Determine whether the event is a full or sync
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
