//! Enums for heterogeneous collections of events, inclusive for every event type that implements
//! the trait of the same name.

use CustomEvent;
use CustomRoomEvent;
use CustomStateEvent;
use call::answer::AnswerEvent;
use call::candidates::CandidatesEvent;
use call::hangup::HangupEvent;
use call::invite::InviteEvent;
use presence::PresenceEvent;
use receipt::ReceiptEvent;
use room::aliases::AliasesEvent;
use room::avatar::AvatarEvent;
use room::canonical_alias::CanonicalAliasEvent;
use room::create::CreateEvent;
use room::guest_access::GuestAccessEvent;
use room::history_visibility::HistoryVisibilityEvent;
use room::join_rules::JoinRulesEvent;
use room::member::MemberEvent;
use room::message::MessageEvent;
use room::name::NameEvent;
use room::power_levels::PowerLevelsEvent;
use room::redaction::RedactionEvent;
use room::third_party_invite::ThirdPartyInviteEvent;
use room::topic::TopicEvent;
use tag::TagEvent;
use typing::TypingEvent;

/// A basic event, room event, or state event.
pub enum Event {
    /// m.call.answer
    CallAnswer(AnswerEvent),
    /// m.call.candidates
    CallCandidates(CandidatesEvent),
    /// m.call.hangup
    CallHangup(HangupEvent),
    /// m.call.invite
    CallInvite(InviteEvent),
    /// m.presence
    Presence(PresenceEvent),
    /// m.receipt
    Receipt(ReceiptEvent),
    /// m.room.aliases
    RoomAliases(AliasesEvent),
    /// m.room.avatar
    RoomAvatar(AvatarEvent),
    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),
    /// m.room.create
    RoomCreate(CreateEvent),
    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),
    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),
    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),
    /// m.room.member
    RoomMember(MemberEvent),
    /// m.room.message
    RoomMessage(MessageEvent),
    /// m.room.name
    RoomName(NameEvent),
    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),
    /// m.room.redaction
    RoomRedaction(RedactionEvent),
    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),
    /// m.room.topic
    RoomTopic(TopicEvent),
    /// m.tag
    Tag(TagEvent),
    /// m.typing
    Typing(TypingEvent),
    /// Any basic event that is not part of the specification.
    Custom(CustomEvent),
    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),
    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
}

/// A room event or state event.
pub enum RoomEvent {
    /// m.call.answer
    CallAnswer(AnswerEvent),
    /// m.call.candidates
    CallCandidates(CandidatesEvent),
    /// m.call.hangup
    CallHangup(HangupEvent),
    /// m.call.invite
    CallInvite(InviteEvent),
    /// m.room.aliases
    RoomAliases(AliasesEvent),
    /// m.room.avatar
    RoomAvatar(AvatarEvent),
    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),
    /// m.room.create
    RoomCreate(CreateEvent),
    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),
    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),
    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),
    /// m.room.member
    RoomMember(MemberEvent),
    /// m.room.message
    RoomMessage(MessageEvent),
    /// m.room.name
    RoomName(NameEvent),
    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),
    /// m.room.redaction
    RoomRedaction(RedactionEvent),
    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),
    /// m.room.topic
    RoomTopic(TopicEvent),
    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),
    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
}

/// A state event.
pub enum StateEvent {
    /// m.room.aliases
    RoomAliases(AliasesEvent),
    /// m.room.avatar
    RoomAvatar(AvatarEvent),
    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),
    /// m.room.create
    RoomCreate(CreateEvent),
    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),
    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),
    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),
    /// m.room.member
    RoomMember(MemberEvent),
    /// m.room.name
    RoomName(NameEvent),
    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),
    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),
    /// m.room.topic
    RoomTopic(TopicEvent),
    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
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

impl_from_t_for_event!(AnswerEvent, CallAnswer);
impl_from_t_for_event!(CandidatesEvent, CallCandidates);
impl_from_t_for_event!(HangupEvent, CallHangup);
impl_from_t_for_event!(InviteEvent, CallInvite);
impl_from_t_for_event!(PresenceEvent, Presence);
impl_from_t_for_event!(ReceiptEvent, Receipt);
impl_from_t_for_event!(AliasesEvent, RoomAliases);
impl_from_t_for_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_event!(CreateEvent, RoomCreate);
impl_from_t_for_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_event!(MemberEvent, RoomMember);
impl_from_t_for_event!(MessageEvent, RoomMessage);
impl_from_t_for_event!(NameEvent, RoomName);
impl_from_t_for_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_event!(TopicEvent, RoomTopic);
impl_from_t_for_event!(TagEvent, Tag);
impl_from_t_for_event!(TypingEvent, Typing);
impl_from_t_for_event!(CustomEvent, Custom);
impl_from_t_for_event!(CustomRoomEvent, CustomRoom);
impl_from_t_for_event!(CustomStateEvent, CustomState);

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
impl_from_t_for_room_event!(AliasesEvent, RoomAliases);
impl_from_t_for_room_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_room_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_room_event!(CreateEvent, RoomCreate);
impl_from_t_for_room_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_room_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_room_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_room_event!(MemberEvent, RoomMember);
impl_from_t_for_room_event!(MessageEvent, RoomMessage);
impl_from_t_for_room_event!(NameEvent, RoomName);
impl_from_t_for_room_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_room_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_room_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_room_event!(TopicEvent, RoomTopic);
impl_from_t_for_room_event!(CustomRoomEvent, CustomRoom);
impl_from_t_for_room_event!(CustomStateEvent, CustomState);

macro_rules! impl_from_t_for_state_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for StateEvent {
            fn from(event: $ty) -> Self {
                StateEvent::$variant(event)
            }
        }
    };
}

impl_from_t_for_state_event!(AliasesEvent, RoomAliases);
impl_from_t_for_state_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_state_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_state_event!(CreateEvent, RoomCreate);
impl_from_t_for_state_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_state_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_state_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_state_event!(MemberEvent, RoomMember);
impl_from_t_for_state_event!(NameEvent, RoomName);
impl_from_t_for_state_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_state_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_state_event!(TopicEvent, RoomTopic);
impl_from_t_for_state_event!(CustomStateEvent, CustomState);
