//! Enums for heterogeneous collections of events, exclusive to event types that implement "at
//! most" the trait of the same name.

use serde::{de::Error as _, Deserialize, Deserializer};
use serde_json::Value;

pub use super::all::StateEvent;
use crate::{
    call::{
        answer::raw::AnswerEvent, candidates::raw::CandidatesEvent, hangup::raw::HangupEvent,
        invite::raw::InviteEvent,
    },
    custom::raw::{CustomEvent, CustomRoomEvent},
    direct::raw::DirectEvent,
    dummy::raw::DummyEvent,
    forwarded_room_key::raw::ForwardedRoomKeyEvent,
    fully_read::raw::FullyReadEvent,
    ignored_user_list::raw::IgnoredUserListEvent,
    key::verification::{
        accept::raw::AcceptEvent, cancel::raw::CancelEvent, key::raw::KeyEvent, mac::raw::MacEvent,
        request::raw::RequestEvent, start::raw::StartEvent,
    },
    presence::raw::PresenceEvent,
    push_rules::raw::PushRulesEvent,
    receipt::raw::ReceiptEvent,
    room::{
        encrypted::raw::EncryptedEvent,
        message::{feedback::raw::FeedbackEvent, raw::MessageEvent},
        redaction::raw::RedactionEvent,
    },
    room_key::raw::RoomKeyEvent,
    room_key_request::raw::RoomKeyRequestEvent,
    sticker::raw::StickerEvent,
    tag::raw::TagEvent,
    typing::raw::TypingEvent,
    util::get_field,
    EventType,
};

/// A basic event.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
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

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::util::try_variant_from_value as from_value;
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_field(&value, "type")?;

        match event_type {
            Direct => from_value(value, Event::Direct),
            Dummy => from_value(value, Event::Dummy),
            ForwardedRoomKey => from_value(value, Event::ForwardedRoomKey),
            FullyRead => from_value(value, Event::FullyRead),
            KeyVerificationAccept => from_value(value, Event::KeyVerificationAccept),
            KeyVerificationCancel => from_value(value, Event::KeyVerificationCancel),
            KeyVerificationKey => from_value(value, Event::KeyVerificationKey),
            KeyVerificationMac => from_value(value, Event::KeyVerificationMac),
            KeyVerificationRequest => from_value(value, Event::KeyVerificationRequest),
            KeyVerificationStart => from_value(value, Event::KeyVerificationStart),
            IgnoredUserList => from_value(value, Event::IgnoredUserList),
            Presence => from_value(value, Event::Presence),
            PushRules => from_value(value, Event::PushRules),
            RoomKey => from_value(value, Event::RoomKey),
            RoomKeyRequest => from_value(value, Event::RoomKeyRequest),
            Receipt => from_value(value, Event::Receipt),
            Tag => from_value(value, Event::Tag),
            Typing => from_value(value, Event::Typing),
            Custom(_event_type_name) => from_value(value, Event::Custom),
            CallAnswer
            | CallCandidates
            | CallHangup
            | CallInvite
            | RoomAliases
            | RoomAvatar
            | RoomCanonicalAlias
            | RoomCreate
            | RoomEncrypted
            | RoomEncryption
            | RoomGuestAccess
            | RoomHistoryVisibility
            | RoomJoinRules
            | RoomMember
            | RoomMessage
            | RoomMessageFeedback
            | RoomName
            | RoomPinnedEvents
            | RoomPowerLevels
            | RoomServerAcl
            | RoomThirdPartyInvite
            | RoomTombstone
            | RoomTopic
            | RoomRedaction
            | Sticker => Err(D::Error::custom("invalid event type")),
        }
    }
}

impl<'de> Deserialize<'de> for RoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::util::try_variant_from_value as from_value;
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_field(&value, "type")?;

        match event_type {
            CallAnswer => from_value(value, RoomEvent::CallAnswer),
            CallCandidates => from_value(value, RoomEvent::CallCandidates),
            CallHangup => from_value(value, RoomEvent::CallHangup),
            CallInvite => from_value(value, RoomEvent::CallInvite),
            RoomEncrypted => from_value(value, RoomEvent::RoomEncrypted),
            RoomMessage => from_value(value, RoomEvent::RoomMessage),
            RoomMessageFeedback => from_value(value, RoomEvent::RoomMessageFeedback),
            RoomRedaction => from_value(value, RoomEvent::RoomRedaction),
            Sticker => from_value(value, RoomEvent::Sticker),
            Custom(_event_type_name) => from_value(value, RoomEvent::CustomRoom),
            Direct
            | Dummy
            | ForwardedRoomKey
            | FullyRead
            | IgnoredUserList
            | KeyVerificationAccept
            | KeyVerificationCancel
            | KeyVerificationKey
            | KeyVerificationMac
            | KeyVerificationRequest
            | KeyVerificationStart
            | Presence
            | PushRules
            | Receipt
            | RoomAvatar
            | RoomAliases
            | RoomCanonicalAlias
            | RoomCreate
            | RoomEncryption
            | RoomGuestAccess
            | RoomHistoryVisibility
            | RoomJoinRules
            | RoomKey
            | RoomKeyRequest
            | RoomMember
            | RoomName
            | RoomPinnedEvents
            | RoomPowerLevels
            | RoomServerAcl
            | RoomThirdPartyInvite
            | RoomTombstone
            | RoomTopic
            | Tag
            | Typing => Err(D::Error::custom("invalid event type")),
        }
    }
}
