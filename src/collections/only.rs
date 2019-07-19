//! Enums for heterogeneous collections of events, exclusive to event types that implement "at
//! most" the trait of the same name.

use std::{convert::TryFrom, str::FromStr};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

pub use super::all::StateEvent;
use crate::{
    call::{
        answer::AnswerEvent, candidates::CandidatesEvent, hangup::HangupEvent, invite::InviteEvent,
    },
    direct::DirectEvent,
    dummy::DummyEvent,
    forwarded_room_key::ForwardedRoomKeyEvent,
    fully_read::FullyReadEvent,
    ignored_user_list::IgnoredUserListEvent,
    key::verification::{
        accept::AcceptEvent, cancel::CancelEvent, key::KeyEvent, mac::MacEvent,
        request::RequestEvent, start::StartEvent,
    },
    presence::PresenceEvent,
    push_rules::PushRulesEvent,
    receipt::ReceiptEvent,
    room::{
        encrypted::EncryptedEvent,
        message::{feedback::FeedbackEvent, MessageEvent},
        redaction::RedactionEvent,
    },
    room_key::RoomKeyEvent,
    room_key_request::RoomKeyRequestEvent,
    sticker::StickerEvent,
    tag::TagEvent,
    typing::TypingEvent,
    CustomEvent, CustomRoomEvent, EventType, InnerInvalidEvent, InvalidEvent,
};

/// A basic event.
#[derive(Clone, Debug)]
pub enum Event {
    /// m.direct
    Direct(DirectEvent),

    /// m.dummy
    Dummy(DummyEvent),

    /// m.forwarded_room_key
    ForwardedRoomKey(ForwardedRoomKeyEvent),

    /// m.fully_read
    FullyRead(FullyReadEvent),

    /// m.key.verification.accept
    KeyVerificationAccept(AcceptEvent),

    /// m.key.verification.cancel
    KeyVerificationCancel(CancelEvent),

    /// m.key.verification.key
    KeyVerificationKey(KeyEvent),

    /// m.key.verification.mac
    KeyVerificationMac(MacEvent),

    /// m.key.verification.request
    KeyVerificationRequest(RequestEvent),

    /// m.key.verification.start
    KeyVerificationStart(StartEvent),

    /// m.ignored_user_list
    IgnoredUserList(IgnoredUserListEvent),

    /// m.presence
    Presence(PresenceEvent),

    /// m.push_rules
    PushRules(PushRulesEvent),

    /// m.room_key
    RoomKey(RoomKeyEvent),

    /// m.room_key_request
    RoomKeyRequest(RoomKeyRequestEvent),

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
#[allow(clippy::large_enum_variant)]
pub enum RoomEvent {
    /// m.call.answer
    CallAnswer(AnswerEvent),

    /// m.call.candidates
    CallCandidates(CandidatesEvent),

    /// m.call.hangup
    CallHangup(HangupEvent),

    /// m.call.invite
    CallInvite(InviteEvent),

    /// m.room.encrypted
    RoomEncrypted(EncryptedEvent),

    /// m.room.message
    RoomMessage(MessageEvent),

    /// m.room.message.feedback
    RoomMessageFeedback(FeedbackEvent),

    /// m.room.redaction
    RoomRedaction(RedactionEvent),

    /// m.sticker
    Sticker(StickerEvent),

    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Event::Direct(ref event) => event.serialize(serializer),
            Event::Dummy(ref event) => event.serialize(serializer),
            Event::ForwardedRoomKey(ref event) => event.serialize(serializer),
            Event::FullyRead(ref event) => event.serialize(serializer),
            Event::KeyVerificationAccept(ref event) => event.serialize(serializer),
            Event::KeyVerificationCancel(ref event) => event.serialize(serializer),
            Event::KeyVerificationKey(ref event) => event.serialize(serializer),
            Event::KeyVerificationMac(ref event) => event.serialize(serializer),
            Event::KeyVerificationRequest(ref event) => event.serialize(serializer),
            Event::KeyVerificationStart(ref event) => event.serialize(serializer),
            Event::IgnoredUserList(ref event) => event.serialize(serializer),
            Event::Presence(ref event) => event.serialize(serializer),
            Event::PushRules(ref event) => event.serialize(serializer),
            Event::Receipt(ref event) => event.serialize(serializer),
            Event::RoomKey(ref event) => event.serialize(serializer),
            Event::RoomKeyRequest(ref event) => event.serialize(serializer),
            Event::Tag(ref event) => event.serialize(serializer),
            Event::Typing(ref event) => event.serialize(serializer),
            Event::Custom(ref event) => event.serialize(serializer),
        }
    }
}

impl FromStr for Event {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let value: Value = serde_json::from_str(json)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => {
                return Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: "missing field `type`".to_string(),
                }))
            }
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => {
                return Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                }))
            }
        };

        match event_type {
            EventType::Direct => match json.parse() {
                Ok(event) => Ok(Event::Direct(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Dummy => match json.parse() {
                Ok(event) => Ok(Event::Dummy(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::ForwardedRoomKey => match json.parse() {
                Ok(event) => Ok(Event::ForwardedRoomKey(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::FullyRead => match json.parse() {
                Ok(event) => Ok(Event::FullyRead(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationAccept => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationAccept(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationCancel => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationCancel(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationKey => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationKey(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationMac => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationMac(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationRequest => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationRequest(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::KeyVerificationStart => match json.parse() {
                Ok(event) => Ok(Event::KeyVerificationStart(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::IgnoredUserList => match json.parse() {
                Ok(event) => Ok(Event::IgnoredUserList(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Presence => match json.parse() {
                Ok(event) => Ok(Event::Presence(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::PushRules => match json.parse() {
                Ok(event) => Ok(Event::PushRules(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Receipt => match json.parse() {
                Ok(event) => Ok(Event::Receipt(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomKey => match json.parse() {
                Ok(event) => Ok(Event::RoomKey(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomKeyRequest => match json.parse() {
                Ok(event) => Ok(Event::RoomKeyRequest(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Tag => match json.parse() {
                Ok(event) => Ok(Event::Tag(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Typing => match json.parse() {
                Ok(event) => Ok(Event::Typing(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Custom(_) => match json.parse() {
                Ok(event) => Ok(Event::Custom(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::CallAnswer
            | EventType::CallCandidates
            | EventType::CallHangup
            | EventType::CallInvite
            | EventType::RoomAliases
            | EventType::RoomAvatar
            | EventType::RoomCanonicalAlias
            | EventType::RoomCreate
            | EventType::RoomEncrypted
            | EventType::RoomEncryption
            | EventType::RoomGuestAccess
            | EventType::RoomHistoryVisibility
            | EventType::RoomJoinRules
            | EventType::RoomMember
            | EventType::RoomMessage
            | EventType::RoomMessageFeedback
            | EventType::RoomName
            | EventType::RoomPinnedEvents
            | EventType::RoomPowerLevels
            | EventType::RoomRedaction
            | EventType::RoomServerAcl
            | EventType::RoomThirdPartyInvite
            | EventType::RoomTombstone
            | EventType::RoomTopic
            | EventType::Sticker => Err(InvalidEvent(InnerInvalidEvent::Validation {
                json: value,
                message: "not exclusively a basic event".to_string(),
            })),
            EventType::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
            }
        }
    }
}

impl Serialize for RoomEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            RoomEvent::CallAnswer(ref event) => event.serialize(serializer),
            RoomEvent::CallCandidates(ref event) => event.serialize(serializer),
            RoomEvent::CallHangup(ref event) => event.serialize(serializer),
            RoomEvent::CallInvite(ref event) => event.serialize(serializer),
            RoomEvent::RoomEncrypted(ref event) => event.serialize(serializer),
            RoomEvent::RoomMessage(ref event) => event.serialize(serializer),
            RoomEvent::RoomMessageFeedback(ref event) => event.serialize(serializer),
            RoomEvent::RoomRedaction(ref event) => event.serialize(serializer),
            RoomEvent::Sticker(ref event) => event.serialize(serializer),
            RoomEvent::CustomRoom(ref event) => event.serialize(serializer),
        }
    }
}

impl FromStr for RoomEvent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let value: Value = serde_json::from_str(json)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => {
                return Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: "missing field `type`".to_string(),
                }))
            }
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => {
                return Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                }))
            }
        };

        match event_type {
            EventType::CallAnswer => match json.parse() {
                Ok(event) => Ok(RoomEvent::CallAnswer(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::CallCandidates => match json.parse() {
                Ok(event) => Ok(RoomEvent::CallCandidates(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::CallHangup => match json.parse() {
                Ok(event) => Ok(RoomEvent::CallHangup(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::CallInvite => match json.parse() {
                Ok(event) => Ok(RoomEvent::CallInvite(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomEncrypted => match json.parse() {
                Ok(event) => Ok(RoomEvent::RoomEncrypted(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomMessage => match json.parse() {
                Ok(event) => Ok(RoomEvent::RoomMessage(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomMessageFeedback => match json.parse() {
                Ok(event) => Ok(RoomEvent::RoomMessageFeedback(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::RoomRedaction => match json.parse() {
                Ok(event) => Ok(RoomEvent::RoomRedaction(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Sticker => match json.parse() {
                Ok(event) => Ok(RoomEvent::Sticker(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Custom(_) => match json.parse() {
                Ok(event) => Ok(RoomEvent::CustomRoom(event)),
                Err(error) => Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })),
            },
            EventType::Direct
            | EventType::Dummy
            | EventType::ForwardedRoomKey
            | EventType::FullyRead
            | EventType::KeyVerificationAccept
            | EventType::KeyVerificationCancel
            | EventType::KeyVerificationKey
            | EventType::KeyVerificationMac
            | EventType::KeyVerificationRequest
            | EventType::KeyVerificationStart
            | EventType::IgnoredUserList
            | EventType::Presence
            | EventType::PushRules
            | EventType::Receipt
            | EventType::RoomAliases
            | EventType::RoomAvatar
            | EventType::RoomCanonicalAlias
            | EventType::RoomCreate
            | EventType::RoomEncryption
            | EventType::RoomGuestAccess
            | EventType::RoomHistoryVisibility
            | EventType::RoomJoinRules
            | EventType::RoomMember
            | EventType::RoomName
            | EventType::RoomPinnedEvents
            | EventType::RoomPowerLevels
            | EventType::RoomServerAcl
            | EventType::RoomThirdPartyInvite
            | EventType::RoomTombstone
            | EventType::RoomTopic
            | EventType::RoomKey
            | EventType::RoomKeyRequest
            | EventType::Tag
            | EventType::Typing => Err(InvalidEvent(InnerInvalidEvent::Validation {
                json: value,
                message: "not exclusively a room event".to_string(),
            })),
            EventType::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
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

impl_from_t_for_event!(DirectEvent, Direct);
impl_from_t_for_event!(DummyEvent, Dummy);
impl_from_t_for_event!(ForwardedRoomKeyEvent, ForwardedRoomKey);
impl_from_t_for_event!(FullyReadEvent, FullyRead);
impl_from_t_for_event!(AcceptEvent, KeyVerificationAccept);
impl_from_t_for_event!(CancelEvent, KeyVerificationCancel);
impl_from_t_for_event!(KeyEvent, KeyVerificationKey);
impl_from_t_for_event!(MacEvent, KeyVerificationMac);
impl_from_t_for_event!(RequestEvent, KeyVerificationRequest);
impl_from_t_for_event!(StartEvent, KeyVerificationStart);
impl_from_t_for_event!(IgnoredUserListEvent, IgnoredUserList);
impl_from_t_for_event!(PresenceEvent, Presence);
impl_from_t_for_event!(PushRulesEvent, PushRules);
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
impl_from_t_for_room_event!(EncryptedEvent, RoomEncrypted);
impl_from_t_for_room_event!(MessageEvent, RoomMessage);
impl_from_t_for_room_event!(FeedbackEvent, RoomMessageFeedback);
impl_from_t_for_room_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_room_event!(StickerEvent, Sticker);
impl_from_t_for_room_event!(CustomRoomEvent, CustomRoom);
