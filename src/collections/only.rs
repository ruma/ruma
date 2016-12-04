//! Enums for heterogeneous collections of events, exclusive to event types that implement "at
//! most" the trait of the same name.

use CustomEvent;
use CustomRoomEvent;
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

pub use super::all::StateEvent;

/// A basic event.
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
