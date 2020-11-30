use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma::{
    events::{
        from_raw_json_value,
        pdu::{EventHash, Pdu, PduStub},
        room::member::{MemberEventContent, MembershipState},
        EventDeHelper, EventType,
    },
    serde::CanonicalJsonValue,
    signatures::reference_hash,
    EventId, RoomId, RoomVersionId, ServerName, UserId,
};
use serde::{de, ser, Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EventIdHelper {
    event_id: EventId,
}

/// This feature is turned on in conduit but off when the tests run because
/// we rely on the EventId to check the state-res.
#[cfg(feature = "gen-eventid")]
fn event_id<E: de::Error>(json: &RawJsonValue) -> Result<EventId, E> {
    use std::convert::TryFrom;
    EventId::try_from(format!(
        "${}",
        reference_hash(&from_raw_json_value(&json)?, &RoomVersionId::Version6)
            .map_err(de::Error::custom)?,
    ))
    .map_err(de::Error::custom)
}

/// Only turned on for testing where we need to keep the ID.
#[cfg(not(feature = "gen-eventid"))]
fn event_id<E: de::Error>(json: &RawJsonValue) -> Result<EventId, E> {
    use std::convert::TryFrom;
    Ok(match from_raw_json_value::<EventIdHelper, E>(&json) {
        Ok(id) => id.event_id,
        Err(_) => {
            // panic!("NOT DURING TESTS");
            EventId::try_from(format!(
                "${}",
                reference_hash(&from_raw_json_value(&json)?, &RoomVersionId::Version6)
                    .map_err(de::Error::custom)?,
            ))
            .map_err(de::Error::custom)?
        }
    })
}

pub struct Requester<'a> {
    pub prev_event_ids: Vec<EventId>,
    pub room_id: &'a RoomId,
    pub content: &'a serde_json::Value,
    pub state_key: Option<String>,
    pub sender: &'a UserId,
}

#[derive(Clone, Debug)]
pub enum StateEvent {
    Full(EventId, Pdu),
    Stub(PduStub),
}

impl Serialize for StateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::Error;
        use std::convert::TryInto;

        match self {
            Self::Full(id, ev) => {
                // TODO: do we want to add the eventId when we
                // serialize
                let val: CanonicalJsonValue = serde_json::to_value(ev)
                    .map_err(S::Error::custom)?
                    .try_into()
                    .map_err(S::Error::custom)?;

                match val {
                    CanonicalJsonValue::Object(mut obj) => {
                        obj.insert(
                            "event_id".into(),
                            ruma::serde::to_canonical_value(id).map_err(S::Error::custom)?,
                        );
                        obj.serialize(serializer)
                    }
                    _ => panic!("Pdu not an object"),
                }
            }
            Self::Stub(_) => panic!("Found PduStub"),
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

        // TODO: do we even want to try for the existing ID

        // Determine whether the event is a full or stub
        // based on the fields present.
        Ok(if room_id.is_some() {
            match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Full(
                    event_id(&json)?,
                    Pdu::RoomV3Pdu(from_raw_json_value(&json)?),
                ),
            }
        } else {
            match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Stub(from_raw_json_value(&json)?),
            }
        })
    }
}

impl StateEvent {
    pub fn from_id_value(id: EventId, json: serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Self::Full(
            id,
            Pdu::RoomV3Pdu(serde_json::from_value(json)?),
        ))
    }

    pub fn from_id_canon_obj(
        id: EventId,
        json: ruma::serde::CanonicalJsonObject,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::Full(
            id,
            // TODO: this is unfortunate (from_value(to_value(json)))...
            Pdu::RoomV3Pdu(serde_json::from_value(serde_json::to_value(json)?)?),
        ))
    }

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
            Self::Full(_, any_event) => match any_event {
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
            Self::Stub(any_event) => match any_event {
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
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => serde_json::from_value(ev.content.clone()),
                Pdu::RoomV3Pdu(ev) => serde_json::from_value(ev.content.clone()),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => serde_json::from_value(ev.content.clone()),
                PduStub::RoomV3PduStub(ev) => serde_json::from_value(ev.content.clone()),
            },
        }
    }
    pub fn origin_server_ts(&self) -> &SystemTime {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.origin_server_ts,
                Pdu::RoomV3Pdu(ev) => &ev.origin_server_ts,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.origin_server_ts,
                PduStub::RoomV3PduStub(ev) => &ev.origin_server_ts,
            },
        }
    }
    pub fn event_id(&self) -> EventId {
        match self {
            // TODO; make this a &EventId
            Self::Full(id, _) => id.clone(),
            Self::Stub(_) => panic!("Stubs don't have an event id"),
        }
    }

    pub fn sender(&self) -> &UserId {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.sender,
                Pdu::RoomV3Pdu(ev) => &ev.sender,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.sender,
                PduStub::RoomV3PduStub(ev) => &ev.sender,
            },
        }
    }

    pub fn redacts(&self) -> Option<&EventId> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.redacts.as_ref(),
                PduStub::RoomV3PduStub(ev) => ev.redacts.as_ref(),
            },
        }
    }

    pub fn room_id(&self) -> Option<&RoomId> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => Some(&ev.room_id),
                Pdu::RoomV3Pdu(ev) => Some(&ev.room_id),
            },
            Self::Stub(_) => None,
        }
    }
    pub fn kind(&self) -> EventType {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.kind.clone(),
                Pdu::RoomV3Pdu(ev) => ev.kind.clone(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.kind.clone(),
                PduStub::RoomV3PduStub(ev) => ev.kind.clone(),
            },
        }
    }
    pub fn state_key(&self) -> String {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.state_key.clone(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.clone(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.state_key.clone(),
                PduStub::RoomV3PduStub(ev) => ev.state_key.clone(),
            },
        }
        .expect("All state events have a state key")
    }

    #[cfg(not(feature = "unstable-pre-spec"))]
    pub fn origin(&self) -> String {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.origin.clone(),
                Pdu::RoomV3Pdu(ev) => ev.origin.clone(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.origin.clone(),
                PduStub::RoomV3PduStub(ev) => ev.origin.clone(),
            },
        }
    }

    pub fn prev_event_ids(&self) -> Vec<EventId> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.prev_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.prev_events.clone(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.prev_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.prev_events.to_vec(),
            },
        }
    }

    pub fn auth_events(&self) -> Vec<EventId> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => ev.auth_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.auth_events.to_vec(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => {
                    ev.auth_events.iter().map(|(id, _)| id).cloned().collect()
                }
                PduStub::RoomV3PduStub(ev) => ev.auth_events.to_vec(),
            },
        }
    }

    pub fn content(&self) -> &serde_json::Value {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.content,
                Pdu::RoomV3Pdu(ev) => &ev.content,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.content,
                PduStub::RoomV3PduStub(ev) => &ev.content,
            },
        }
    }

    pub fn unsigned(&self) -> &BTreeMap<String, serde_json::Value> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.unsigned,
                Pdu::RoomV3Pdu(ev) => &ev.unsigned,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.unsigned,
                PduStub::RoomV3PduStub(ev) => &ev.unsigned,
            },
        }
    }

    pub fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ruma::ServerKeyId, String>> {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(_) => maplit::btreemap! {},
                Pdu::RoomV3Pdu(ev) => ev.signatures.clone(),
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => ev.signatures.clone(),
                PduStub::RoomV3PduStub(ev) => ev.signatures.clone(),
            },
        }
    }

    pub fn hashes(&self) -> &EventHash {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.hashes,
                Pdu::RoomV3Pdu(ev) => &ev.hashes,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.hashes,
                PduStub::RoomV3PduStub(ev) => &ev.hashes,
            },
        }
    }

    pub fn depth(&self) -> &UInt {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => &ev.depth,
                Pdu::RoomV3Pdu(ev) => &ev.depth,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(ev) => &ev.depth,
                PduStub::RoomV3PduStub(ev) => &ev.depth,
            },
        }
    }

    pub fn is_type_and_key(&self, ev_type: EventType, state_key: &str) -> bool {
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
                Pdu::RoomV3Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
            },
            Self::Stub(ev) => match ev {
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
    /// Currently either version 1 or 6 is returned, 6 represents
    /// version 3 and above.
    pub fn room_version(&self) -> RoomVersionId {
        // TODO: We have to know the actual room version this is not sufficient
        match self {
            Self::Full(_, ev) => match ev {
                Pdu::RoomV1Pdu(_) => RoomVersionId::Version1,
                Pdu::RoomV3Pdu(_) => RoomVersionId::Version6,
            },
            Self::Stub(ev) => match ev {
                PduStub::RoomV1PduStub(_) => RoomVersionId::Version1,
                PduStub::RoomV3PduStub(_) => RoomVersionId::Version6,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_pdu() {
        let non_canonical_json = r#"{"auth_events": ["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE", "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY", "$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"], "content": {"membership": "join"}, "depth": 135, "hashes": {"sha256": "Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"}, "origin": "pc.koesters.xyz:59003", "origin_server_ts": 1599901756522, "prev_events": ["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"], "prev_state": [], "room_id": "!eGNyCFvnKcpsnIZiEV:koesters.xyz", "sender": "@timo:pc.koesters.xyz:59003", "state_key": "@timo:pc.koesters.xyz:59003", "type": "m.room.member", "signatures": {"koesters.xyz": {"ed25519:a_wwQy": "bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"}, "pc.koesters.xyz:59003": {"ed25519:key1": "/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}}, "unsigned": {"age": 30, "replaces_state": "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "prev_content": {"membership": "join"}, "prev_sender": "@timo:pc.koesters.xyz:59003"}}"#;

        let pdu = serde_json::from_str::<StateEvent>(non_canonical_json).unwrap();

        assert_eq!(
            match &pdu {
                StateEvent::Full(id, _) => id,
                _ => panic!("Stub found"),
            },
            &ruma::event_id!("$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU")
        );
    }

    #[test]
    fn serialize_pdu() {
        let non_canonical_json = r#"{"auth_events": ["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE", "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY", "$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"], "content": {"membership": "join"}, "depth": 135, "hashes": {"sha256": "Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"}, "origin": "pc.koesters.xyz:59003", "origin_server_ts": 1599901756522, "prev_events": ["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"], "prev_state": [], "room_id": "!eGNyCFvnKcpsnIZiEV:koesters.xyz", "sender": "@timo:pc.koesters.xyz:59003", "state_key": "@timo:pc.koesters.xyz:59003", "type": "m.room.member", "signatures": {"koesters.xyz": {"ed25519:a_wwQy": "bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"}, "pc.koesters.xyz:59003": {"ed25519:key1": "/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}}, "unsigned": {"age": 30, "replaces_state": "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "prev_content": {"membership": "join"}, "prev_sender": "@timo:pc.koesters.xyz:59003"}}"#;

        let pdu = serde_json::from_str::<StateEvent>(non_canonical_json).unwrap();

        assert_eq!(
            match &pdu {
                StateEvent::Full(id, _) => id,
                _ => panic!("Stub found"),
            },
            &ruma::event_id!("$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU")
        );

        // TODO: the `origin` field is left out, though it seems it should be part of the eventId hashing
        // For testing we must serialize the PDU with the `event_id` field this is probably not correct for production
        // although without them we get "Invalid bytes in DB" errors in conduit
        assert_eq!(
            ruma::serde::to_canonical_json_string(&pdu).unwrap(),
            r#"{"auth_events":["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE","$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc","$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY","$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"],"content":{"membership":"join"},"depth":135,"event_id":"$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU","hashes":{"sha256":"Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"},"origin_server_ts":1599901756522,"prev_events":["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"],"room_id":"!eGNyCFvnKcpsnIZiEV:koesters.xyz","sender":"@timo:pc.koesters.xyz:59003","signatures":{"koesters.xyz":{"ed25519:a_wwQy":"bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"},"pc.koesters.xyz:59003":{"ed25519:key1":"/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}},"state_key":"@timo:pc.koesters.xyz:59003","type":"m.room.member","unsigned":{"age":30,"prev_content":{"membership":"join"},"prev_sender":"@timo:pc.koesters.xyz:59003","replaces_state":"$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"}}"#,
        )
    }
}
