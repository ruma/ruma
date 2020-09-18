use std::collections::BTreeMap;

use js_int::UInt;
use ruma::{
    events::{
        pdu::{EventHash, Pdu, PduStub},
        room::member::{MemberEventContent, MembershipState},
        EventType,
    },
    EventId, RoomId, RoomVersionId, ServerName, UserId,
};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

pub struct Requester<'a> {
    pub prev_event_ids: Vec<EventId>,
    pub room_id: &'a RoomId,
    pub content: &'a serde_json::Value,
    pub state_key: Option<String>,
    pub sender: &'a UserId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateEvent {
    Full(Pdu),
    #[serde(skip_deserializing)]
    Sync(PduStub),
}

impl StateEvent {
    pub fn to_requester(&self) -> Requester<'_> {
        Requester {
            prev_event_ids: self.prev_event_ids(),
            room_id: self.room_id().unwrap(),
            content: self.content(),
            state_key: Some(self.state_key()),
            sender: self.sender(),
        }
    }

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
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.event_id,
                Pdu::RoomV3Pdu(ev) => ev.event_id.as_ref().expect("RoomV3Pdu did not have an event id"),
            },
            Self::Sync(_ev) => panic!("Stubs don't have an event id"),
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
    pub fn state_key(&self) -> String {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.state_key.clone(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.state_key.clone(),
                PduStub::RoomV3PduStub(ev) => ev.state_key.clone(),
            },
        }.expect("All state events have a state key")
    }

    #[cfg(not(feature = "unstable-pre-spec"))]
    pub fn origin(&self) -> String {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.origin.clone(),
                Pdu::RoomV3Pdu(ev) => ev.origin.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.origin.clone(),
                PduStub::RoomV3PduStub(ev) => ev.origin.clone(),
            },
        }
    }

    pub fn prev_event_ids(&self) -> Vec<EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.prev_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.prev_events.clone(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.prev_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.prev_events.to_vec(),
            },
        }
    }

    pub fn auth_events(&self) -> Vec<EventId> {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.auth_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.auth_events.to_vec(),
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.auth_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.auth_events.to_vec(),
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

    pub fn unsigned(&self) -> &BTreeMap<String, serde_json::Value> {
        // CONFIRM: The only way this would fail is if we got bad json, it should fail in ruma
        // before it fails here.
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.unsigned,
                Pdu::RoomV3Pdu(ev) => &ev.unsigned,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.unsigned,
                PduStub::RoomV3PduStub(ev) => &ev.unsigned,
            },
        }
    }

    pub fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ruma::ServerKeyId, String>> {
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

    pub fn hashes(&self) -> &EventHash {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.hashes,
                Pdu::RoomV3Pdu(ev) => &ev.hashes,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.hashes,
                PduStub::RoomV3PduStub(ev) => &ev.hashes,
            },
        }
    }

    pub fn depth(&self) -> &UInt {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.depth,
                Pdu::RoomV3Pdu(ev) => &ev.depth,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.depth,
                PduStub::RoomV3PduStub(ev) => &ev.depth,
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

    /// Returns the room version this event is formatted for.
    ///
    /// Currently either version 1 or 3 is returned, 3 represents
    /// version 3 and above.
    pub fn room_version(&self) -> RoomVersionId {
        match self {
            Self::Full(ev) => match ev {
                Pdu::RoomV1Pdu(_) => RoomVersionId::Version1,
                Pdu::RoomV3Pdu(_) => RoomVersionId::Version3,
            },
            Self::Sync(ev) => match ev {
                PduStub::RoomV1PduStub(_) => RoomVersionId::Version1,
                PduStub::RoomV3PduStub(_) => RoomVersionId::Version3,
            },
        }
    }
}
