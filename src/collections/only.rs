//! Enums for heterogeneous collections of events, exclusive to event types that implement "at
//! most" the trait of the same name.

use {CustomEvent, CustomRoomEvent, EventType};
use call::answer::AnswerEvent;
use call::candidates::CandidatesEvent;
use call::hangup::HangupEvent;
use call::invite::InviteEvent;
use presence::PresenceEvent;
use receipt::ReceiptEvent;
use room::message::MessageEvent;
use room::redaction::RedactionEvent;
use tag::TagEvent;
use typing::TypingEvent;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde_json::{Value, from_value};

pub use super::all::StateEvent;

/// A basic event.
#[derive(Clone, Debug)]
pub enum Event {
    /// m.presence
    Presence(PresenceEvent),
    /// m.receipt
    Receipt(ReceiptEvent),
    /// m.tag
    Tag(TagEvent),
    /// m.typing
    Typing(TypingEvent),
    /// Any basic event that is not part of the specification.
    Custom(CustomEvent),
}

/// A room event.
#[derive(Clone, Debug)]
pub enum RoomEvent {
    /// m.call.answer
    CallAnswer(AnswerEvent),
    /// m.call.candidates
    CallCandidates(CandidatesEvent),
    /// m.call.hangup
    CallHangup(HangupEvent),
    /// m.call.invite
    CallInvite(InviteEvent),
    /// m.room.message
    RoomMessage(MessageEvent),
    /// m.room.redaction
    RoomRedaction(RedactionEvent),
    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match *self {
            Event::Presence(ref event) => event.serialize(serializer),
            Event::Receipt(ref event) => event.serialize(serializer),
            Event::Tag(ref event) => event.serialize(serializer),
            Event::Typing(ref event) => event.serialize(serializer),
            Event::Custom(ref event) => event.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value: Value = try!(Deserialize::deserialize(deserializer));

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::Presence => {
                let event = match from_value::<PresenceEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Presence(event))
            }
            EventType::Receipt => {
                let event = match from_value::<ReceiptEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Receipt(event))
            }
            EventType::Tag => {
                let event = match from_value::<TagEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Tag(event))
            }
            EventType::Typing => {
                let event = match from_value::<TypingEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Typing(event))
            }
            EventType::Custom(_) => {
                let event = match from_value::<CustomEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Custom(event))
            }
            EventType::CallAnswer | EventType::CallCandidates | EventType::CallHangup |
            EventType::CallInvite | EventType::RoomAliases | EventType::RoomAvatar |
            EventType::RoomCanonicalAlias | EventType::RoomCreate | EventType::RoomGuestAccess |
            EventType::RoomHistoryVisibility | EventType::RoomJoinRules | EventType::RoomMember |
            EventType::RoomMessage | EventType::RoomName | EventType::RoomPowerLevels |
            EventType::RoomRedaction | EventType::RoomThirdPartyInvite | EventType::RoomTopic => {
                return Err(D::Error::custom("not exclusively a basic event".to_string()));
            }
        }
    }
}

impl Serialize for RoomEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match *self {
            RoomEvent::CallAnswer(ref event) => event.serialize(serializer),
            RoomEvent::CallCandidates(ref event) => event.serialize(serializer),
            RoomEvent::CallHangup(ref event) => event.serialize(serializer),
            RoomEvent::CallInvite(ref event) => event.serialize(serializer),
            RoomEvent::RoomMessage(ref event) => event.serialize(serializer),
            RoomEvent::RoomRedaction(ref event) => event.serialize(serializer),
            RoomEvent::CustomRoom(ref event) => event.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value: Value = try!(Deserialize::deserialize(deserializer));

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::CallAnswer => {
                let event = match from_value::<AnswerEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallAnswer(event))
            }
            EventType::CallCandidates => {
                let event = match from_value::<CandidatesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallCandidates(event))
            }
            EventType::CallHangup => {
                let event = match from_value::<HangupEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallHangup(event))
            }
            EventType::CallInvite => {
                let event = match from_value::<InviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallInvite(event))
            }
            EventType::RoomMessage => {
                let event = match from_value::<MessageEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomMessage(event))
            }
            EventType::RoomRedaction => {
                let event = match from_value::<RedactionEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomRedaction(event))
            }
            EventType::Custom(_) => {
                let event = match from_value::<CustomRoomEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CustomRoom(event))
            }
            EventType::Presence | EventType::Receipt | EventType::RoomAliases |
            EventType::RoomAvatar | EventType::RoomCanonicalAlias | EventType::RoomCreate |
            EventType::RoomGuestAccess | EventType::RoomHistoryVisibility |
            EventType::RoomJoinRules | EventType::RoomMember | EventType::RoomName |
            EventType::RoomPowerLevels |EventType::RoomThirdPartyInvite | EventType::RoomTopic |
            EventType::Tag | EventType::Typing => {
                return Err(D::Error::custom("not exclusively a room event".to_string()));
            }
        }
    }
}
macro_rules! impl_from_t_for_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for Event {
            fn from(event: $ty) -> Self {
                Event::$variant(event)
            }
        }
    };
}

impl_from_t_for_event!(PresenceEvent, Presence);
impl_from_t_for_event!(ReceiptEvent, Receipt);
impl_from_t_for_event!(TagEvent, Tag);
impl_from_t_for_event!(TypingEvent, Typing);
impl_from_t_for_event!(CustomEvent, Custom);

macro_rules! impl_from_t_for_room_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for RoomEvent {
            fn from(event: $ty) -> Self {
                RoomEvent::$variant(event)
            }
        }
    };
}

impl_from_t_for_room_event!(AnswerEvent, CallAnswer);
impl_from_t_for_room_event!(CandidatesEvent, CallCandidates);
impl_from_t_for_room_event!(HangupEvent, CallHangup);
impl_from_t_for_room_event!(InviteEvent, CallInvite);
impl_from_t_for_room_event!(MessageEvent, RoomMessage);
impl_from_t_for_room_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_room_event!(CustomRoomEvent, CustomRoom);
