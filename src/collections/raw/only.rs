//! Enums for heterogeneous collections of events, exclusive to event types that implement "at
//! most" the trait of the same name.

use serde::{de::Error as _, Deserialize, Deserializer};
use serde_json::{from_value, Value};

pub use super::all::StateEvent;
use crate::{
    call::{
        answer::raw::AnswerEvent, candidates::raw::CandidatesEvent, hangup::raw::HangupEvent,
        invite::raw::InviteEvent,
    },
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
    util::{get_type_field, serde_json_error_to_generic_de_error as conv_err},
    CustomEvent, CustomRoomEvent, EventType,
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

macro_rules! impl_from_t_for_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for Event {
            fn from(event: $ty) -> Self {
                Event::$variant(event)
            }
        }
    };
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_type_field(&value)?;

        match event_type {
            Direct => from_value(value).map(Self::Direct).map_err(conv_err),
            Dummy => from_value(value).map(Self::Dummy).map_err(conv_err),
            ForwardedRoomKey => from_value(value)
                .map(Self::ForwardedRoomKey)
                .map_err(conv_err),
            FullyRead => from_value(value).map(Self::FullyRead).map_err(conv_err),
            KeyVerificationAccept => from_value(value)
                .map(Self::KeyVerificationAccept)
                .map_err(conv_err),
            KeyVerificationCancel => from_value(value)
                .map(Self::KeyVerificationCancel)
                .map_err(conv_err),
            KeyVerificationKey => from_value(value)
                .map(Self::KeyVerificationKey)
                .map_err(conv_err),
            KeyVerificationMac => from_value(value)
                .map(Self::KeyVerificationMac)
                .map_err(conv_err),
            KeyVerificationRequest => from_value(value)
                .map(Self::KeyVerificationRequest)
                .map_err(conv_err),
            KeyVerificationStart => from_value(value)
                .map(Self::KeyVerificationStart)
                .map_err(conv_err),
            IgnoredUserList => from_value(value)
                .map(Self::IgnoredUserList)
                .map_err(conv_err),
            Presence => from_value(value).map(Self::Presence).map_err(conv_err),
            PushRules => from_value(value).map(Self::PushRules).map_err(conv_err),
            RoomKey => from_value(value).map(Self::RoomKey).map_err(conv_err),
            RoomKeyRequest => from_value(value)
                .map(Self::RoomKeyRequest)
                .map_err(conv_err),
            Receipt => from_value(value).map(Self::Receipt).map_err(conv_err),
            Tag => from_value(value).map(Self::Tag).map_err(conv_err),
            Typing => from_value(value).map(Self::Typing).map_err(conv_err),
            //Custom(_event_type_name) => unimplemented!("todo"),
            _ => Err(D::Error::custom("invalid event type")),
        }
    }
}

impl<'de> Deserialize<'de> for RoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_type_field(&value)?;

        match event_type {
            CallAnswer => from_value(value).map(Self::CallAnswer).map_err(conv_err),
            CallCandidates => from_value(value)
                .map(Self::CallCandidates)
                .map_err(conv_err),
            CallHangup => from_value(value).map(Self::CallHangup).map_err(conv_err),
            CallInvite => from_value(value).map(Self::CallInvite).map_err(conv_err),
            RoomEncrypted => from_value(value).map(Self::RoomEncrypted).map_err(conv_err),
            RoomMessage => from_value(value).map(Self::RoomMessage).map_err(conv_err),
            RoomMessageFeedback => from_value(value)
                .map(Self::RoomMessageFeedback)
                .map_err(conv_err),
            RoomRedaction => from_value(value).map(Self::RoomRedaction).map_err(conv_err),
            Sticker => from_value(value).map(Self::Sticker).map_err(conv_err),
            //Custom(_event_type_name) => unimplemented!("todo"),
            _ => Err(D::Error::custom("invalid event type")),
        }
    }
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
